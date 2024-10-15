use chrono::{DateTime, Utc};
use dotenv::dotenv;
use regex::Regex;
use reqwest::Client;
use rocket::http::{Method, Status};
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::State;
use rocket::{get, launch, routes};
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use serde_json::json;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

const REQUEST_LIMIT: usize = 120;

#[derive(Clone, Default)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<IpAddr, Vec<DateTime<Utc>>>>>,
}

impl RateLimiter {
    fn check_if_rate_limited(&self, ip_addr: IpAddr) -> Result<(), String> {
        let throttle_time_limit = Utc::now() - std::time::Duration::from_secs(10);
        let mut requests_hashmap = self.requests.lock().unwrap();
        let requests_for_ip = requests_hashmap.entry(ip_addr).or_insert(Vec::new());
        requests_for_ip.retain(|x| x > &throttle_time_limit);
        requests_for_ip.push(Utc::now());
        if requests_for_ip.len() > REQUEST_LIMIT {
            return Err("IP is rate limited :(".to_string());
        }
        Ok(())
    }
}

struct ClientIp(IpAddr);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientIp {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(ip) = request.client_ip() {
            Outcome::Success(ClientIp(ip))
        } else {
            Outcome::Error((Status::BadRequest, ()))
        }
    }
}

fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

async fn add_to_brevo_list(email: &str) -> Result<(), String> {
    let client = Client::new();
    let brevo_api_key = std::env::var("BREVO_API_KEY").map_err(|_| "BREVO_API_KEY must be set")?;
    let list_id = 2;

    let response = client
        .post("https://api.brevo.com/v3/contacts")
        .header("api-key", brevo_api_key)
        .json(&json!({
            "email": email,
            "listIds": [list_id],
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "Failed to add email to Brevo list: {}",
            response.status()
        ))
    }
}

#[get("/mail?<email>")]
async fn mail(
    email: String,
    ip: ClientIp,
    rate_limiter: &State<RateLimiter>,
) -> Result<String, Status> {
    if let Err(e) = rate_limiter.check_if_rate_limited(ip.0) {
        return Err(Status::TooManyRequests);
    }

    if !is_valid_email(&email) {
        return Err(Status::BadRequest);
    }

    match add_to_brevo_list(&email).await {
        Ok(_) => Ok("Email added to mailing list successfully!".to_string()),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let cors = rocket_cors::CorsOptions {
        allowed_origins: AllowedOrigins::some_exact(&["https://nodetick.com"]),
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("CORS configuration error");

    rocket::build()
        .mount("/", routes![mail])
        .manage(RateLimiter::default())
        .attach(cors)
}
