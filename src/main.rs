extern crate s3;

use bot::ShipChat;
use dotenv::dotenv;
use headless_chrome::{
    protocol::{page::ScreenshotFormat, target::methods::CreateTarget},
    Browser,
};
use std::env;
use time::{Instant, Duration};

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;

use chrono;
use std::{thread, time};

mod bot;

struct Timeframe {
    parsable_phrases: Vec<String>,
    parsed: String,
}

fn build_timeframe(parsable_phrases: Vec<String>, parsed: String) -> Timeframe {
    Timeframe {
        parsable_phrases: parsable_phrases,
        parsed: parsed,
    }
}

fn parse_timeframe(phrase: String) -> String {
    let timeframes = [
        build_timeframe(
            vec![
                "1".to_string(),
                "1m".to_string(),
                "1min".to_string(),
                "1mins".to_string(),
                "1minute".to_string(),
                "1minutes".to_string(),
                "min".to_string(),
                "m".to_string(),
            ],
            "1".to_string(),
        ),
        build_timeframe(
            vec![
                "3".to_string(),
                "3m".to_string(),
                "3min".to_string(),
                "3mins".to_string(),
                "3minute".to_string(),
                "3minutes".to_string(),
            ],
            "3".to_string(),
        ),
        build_timeframe(
            vec![
                "5".to_string(),
                "5m".to_string(),
                "5min".to_string(),
                "5mins".to_string(),
                "5minute".to_string(),
                "5minutes".to_string(),
            ],
            "5".to_string(),
        ),
        build_timeframe(
            vec![
                "15".to_string(),
                "15m".to_string(),
                "15min".to_string(),
                "15mins".to_string(),
                "15minute".to_string(),
                "15minutes".to_string(),
            ],
            "15".to_string(),
        ),
        build_timeframe(
            vec![
                "30".to_string(),
                "30m".to_string(),
                "30min".to_string(),
                "30mins".to_string(),
                "30minute".to_string(),
                "30minutes".to_string(),
            ],
            "30".to_string(),
        ),
        build_timeframe(
            vec![
                "60".to_string(),
                "60m".to_string(),
                "60min".to_string(),
                "60mins".to_string(),
                "60minute".to_string(),
                "60minutes".to_string(),
                "1".to_string(),
                "1h".to_string(),
                "1hr".to_string(),
                "1hour".to_string(),
                "1hours".to_string(),
                "hourly".to_string(),
                "hour".to_string(),
                "hr".to_string(),
                "h".to_string(),
            ],
            "60".to_string(),
        ),
        build_timeframe(
            vec![
                "120".to_string(),
                "120m".to_string(),
                "120min".to_string(),
                "120mins".to_string(),
                "120minute".to_string(),
                "120minutes".to_string(),
                "2".to_string(),
                "2h".to_string(),
                "2hr".to_string(),
                "2hrs".to_string(),
                "2hour".to_string(),
                "2hours".to_string(),
            ],
            "120".to_string(),
        ),
        build_timeframe(
            vec![
                "180".to_string(),
                "180m".to_string(),
                "180min".to_string(),
                "180mins".to_string(),
                "180minute".to_string(),
                "180minutes".to_string(),
                "3".to_string(),
                "3h".to_string(),
                "3hr".to_string(),
                "3hrs".to_string(),
                "3hour".to_string(),
                "3hours".to_string(),
            ],
            "180".to_string(),
        ),
        build_timeframe(
            vec![
                "240".to_string(),
                "240m".to_string(),
                "240min".to_string(),
                "240mins".to_string(),
                "240minute".to_string(),
                "240minutes".to_string(),
                "4".to_string(),
                "4h".to_string(),
                "4hr".to_string(),
                "4hrs".to_string(),
                "4hour".to_string(),
                "4hours".to_string(),
            ],
            "240".to_string(),
        ),
        build_timeframe(
            vec![
                "24".to_string(),
                "24h".to_string(),
                "24hr".to_string(),
                "24hrs".to_string(),
                "24hour".to_string(),
                "24hours".to_string(),
                "d".to_string(),
                "day".to_string(),
                "1".to_string(),
                "1d".to_string(),
                "1day".to_string(),
                "daily".to_string(),
                "1440".to_string(),
                "1440m".to_string(),
                "1440min".to_string(),
                "1440mins".to_string(),
                "1440minute".to_string(),
                "1440minutes".to_string(),
            ],
            "D".to_string(),
        ),
        build_timeframe(
            vec![
                "7".to_string(),
                "7d".to_string(),
                "7day".to_string(),
                "7days".to_string(),
                "w".to_string(),
                "week".to_string(),
                "1w".to_string(),
                "1week".to_string(),
                "weekly".to_string(),
            ],
            "W".to_string(),
        ),
        build_timeframe(
            vec![
                "30d".to_string(),
                "30day".to_string(),
                "30days".to_string(),
                "1".to_string(),
                "1m".to_string(),
                "m".to_string(),
                "mo".to_string(),
                "month".to_string(),
                "1mo".to_string(),
                "1month".to_string(),
                "monthly".to_string(),
            ],
            "M".to_string(),
        ),
        build_timeframe(
            vec![
                "12".to_string(),
                "12m".to_string(),
                "12mo".to_string(),
                "12month".to_string(),
                "12months".to_string(),
                "year".to_string(),
                "yearly".to_string(),
                "1year".to_string(),
                "1y".to_string(),
                "y".to_string(),
                "annual".to_string(),
                "annually".to_string(),
            ],
            "Y".to_string(),
        ),
    ];

    for tf in timeframes {
        if tf.parsable_phrases.contains(&phrase) {
            return tf.parsed;
        }
    }

    "1".to_string()
}

fn screenshot_tab(url: &str, width: u16, height: u16) -> Result<Vec<u8>, failure::Error> {
    let browser = Browser::default()?;
    let tab = browser.new_tab_with_options(CreateTarget {
        url: url,
        width: Some(width.into()),
        height: Some(height.into()),
        browser_context_id: None,
        enable_begin_frame_control: None,
    })?;
    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;

    tab.wait_for_element(".chart-gui-wrapper > canvas")?;
    let legend = tab.wait_for_element("[data-name='legend-series-item']")?;

    let is_available = legend
        .call_js_fn(
            r#"
        function containsNA () {
            return !this.innerText.includes("n/a");
        }
    "#,
            false,
        )
        .unwrap()
        .value;

    match is_available.eq(&Some(serde_json::value::Value::Bool(true))) {
        true => Ok(tab.capture_screenshot(ScreenshotFormat::PNG, None, true)?),
        false => Err(failure::err_msg("Trading pair not available")),
    }
}

fn setup_s3_bucket() -> Bucket {
    dotenv().ok();

    let credentials: Credentials = Credentials::new(
        Some(&env::var("AWS_ID").unwrap()),
        Some(&env::var("AWS_SECRET").unwrap()),
        None,
        None,
        None,
    )
    .unwrap();
    let region: Region = env::var("S3_REGION").unwrap().parse().unwrap();

    Bucket::new(&env::var("S3_BUCKET").unwrap(), region, credentials).unwrap()
}

fn respond_to_message(authored_message: bot::AuthoredMessage) -> Option<bot::Message> {
    dotenv().ok();
    let now = Instant::now(); // initiate timer
    let width: u16 = "1024".parse().unwrap();
    let height: u16 = "800".parse().unwrap();
    let bucket_name: &String = &env::var("S3_BUCKET").unwrap();
    let region: &String = &env::var("S3_REGION").unwrap();
    println!("Received message: {}", authored_message.contents.to_formatted_string());

    let words = authored_message.contents.to_formatted_words();

    // Error check to ensure sufficient number of words to check for command
    if words.len() <= 2 {
        println!("Error: invalid command");
        if words[0] == "c" {
            return Some(bot::Message::new().add_text(
                "Unknown command.\n
                Type `c <trading_pair> <timeframe>` to get the corresponding chart.\n
                You can look up any trading pair and timeframe supported by TradingView.\n
                Example: `c ethusd 4h`",
            ));
        }

        return None;
    }

    if words[0] == "c" {
        println!("Parsing command.");
        let timeframe: String = parse_timeframe(words[2].to_string());
        let url: String = format!("https://www.tradingview.com/widgetembed/?symbol={}&interval={}&theme=dark&style=1&hidetoptoolbar=1&symboledit=1&saveimage=1&withdateranges=1", words[1], timeframe);
        println!("Getting screenshot from {}", url);
        let shot = screenshot_tab(&url, width, height);
        println!("Got TradingView screenshot, uploading to S3.");
        match shot {
            Ok(n) => {
                let bucket: Bucket = setup_s3_bucket();

                let filename: String = format!(
                    "{}_{}_{:?}.png",
                    words[1],
                    parse_timeframe(timeframe),
                    chrono::offset::Utc::now()
                );

                let (_, _code) = bucket
                    .put_object_with_content_type_blocking(filename.clone(), &n, "image/png")
                    .unwrap();

                let file_location: String = format!(
                    "https://{}.s3.{}.amazonaws.com/{}",
                    bucket_name, region, filename
                );
                println!("Uploaded to S3. Sending URL to chat. Took {} seconds to process command.", now.elapsed().as_secs());

                return Some(bot::Message::new().add_url(file_location.as_str()));
            }
            Err(_err) => {
                let message = format!("Trading pair `{:?}` not available.", words[1]);
                return Some(bot::Message::new().add_text(&message));
            }
        }
    }

    None
}

fn main() {
    // Not used at this time, but ready to be used for Milestone 2
    let mut shipchats: Vec<ShipChat> = Vec::new();
    let shipchat_a = ShipChat {
        ship_name: String::from("~noslur-fabled"),
        chat_name: String::from("chat-8841"),
    };
    let shipchat_b = ShipChat {
        ship_name: String::from("~ristyc-ridwyx"),
        chat_name: String::from("lab-2-9245"),
    };
    shipchats.push(shipchat_b);
    shipchats.push(shipchat_a);

    bot::Chatbot::new_with_local_config(respond_to_message, shipchats).run();
}
