extern crate argparse;
use argparse::{ArgumentParser, Store};
extern crate redis;
use redis::{Client, Commands};
use redis_ts::{TsCommands, TsDuplicatePolicy, TsOptions};
use yahoo_finance_api as yahoo;

fn retrieve_tickers(conn: &mut redis::Connection) -> Vec<String> {
    // Returns a sorted vector containing all the names of the tickers.
    // Each ticker has a key, "attivo" (bool), indicating whether we want to
    // monitor that ticker or we skip it.
    //
    // # Arguments
    // * `conn` - it handles the connection to Redis Timeseries
    //
    // # TODO:
    // * a) errors management and tests
    // * b) eliminate hard-coded redis keys

    let mut tickers_vector: Vec<String> = vec![];
    let answer: Vec<String> = conn.keys("MARKET:METADATA:STOCKS:*").unwrap();

    for x in answer.iter() {
        let status: String = conn.hget(&x, "attivo").unwrap();

        // if active add to the final vector
        if status == "1" {
            let a: Vec<String> = x.split(":").map(|s| s.to_string()).collect();
            tickers_vector.push(a[&a.len() - 1].clone());
        }
    }

    // Sorts it in alphabetical order
    tickers_vector.sort();
    tickers_vector
}

fn get_historical(conn: &mut redis::Connection, tickers: &Vec<String>, period: &mut String) {
    // Returns a sorted vector containing all the names of the tickers.
    // Each ticker has a key, "attivo" (bool), indicating whether we want to
    // monitor that ticker or we skip it.
    //
    // # Arguments
    // * `conn`    - it handles the connection to Redis Timeseries
    // * `tickers` - sorted vector containing the names of the tickers
    // * `period`  - indicates the amount of data we want to retrieve from yahoo finance
    //
    // # TODO:
    // * a) check whether symbol has been delisted before attempting download;
    // * b) add meta-info to redis
    // * c) errors management and tests
    // * d) eliminate hard-coded redis keys

    let provider = yahoo::YahooConnector::new();
    println!("\nI'm about to download the last {period} of historical data for all the tickers\n");

    for (i, t) in tickers.iter().enumerate() {
        println!("Processing {}/{}| {}", i, tickers.len(), t);

        // returns historic quotes with daily interval
        let resp = match tokio_test::block_on(provider.get_quote_range(t, "1d", period)) {
            Ok(resp) => resp,
            Err(error) => {
                println!("# Error processing {t}:\t{error}");
                continue;
            }
        };

        let quotes = resp.quotes().unwrap();

        for qv in quotes.iter() {
            let tms = qv.timestamp as i64 * 1000;

            let redists_options = TsOptions::default().duplicate_policy(TsDuplicatePolicy::Last);
            let _: u64 = conn
                .ts_add_create(
                    format!("MARKET:{}:{}", t, "open"),
                    tms,
                    qv.open as f32,
                    redists_options.clone(),
                )
                .unwrap();
            let _: u64 = conn
                .ts_add_create(
                    format!("MARKET:{}:{}", t, "close"),
                    tms,
                    qv.close as f32,
                    redists_options.clone(),
                )
                .unwrap();
            let _: u64 = conn
                .ts_add_create(
                    format!("MARKET:{}:{}", t, "high"),
                    tms,
                    qv.high as f32,
                    redists_options.clone(),
                )
                .unwrap();
            let _: u64 = conn
                .ts_add_create(
                    format!("MARKET:{}:{}", t, "low"),
                    tms,
                    qv.low as f32,
                    redists_options.clone(),
                )
                .unwrap();
            let _: u64 = conn
                .ts_add_create(
                    format!("MARKET:{}:{}", t, "adj_close"),
                    tms,
                    qv.adjclose as f32,
                    redists_options.clone(),
                )
                .unwrap();
        }
    }
}

fn main() {
    let mut redis_ip = "192.168.1.113".to_string();
    let mut period = "1w".to_string();

    // parsing the external parameters
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Retrieve historical candles data from Yahoo Finance.");
        ap.refer(&mut redis_ip)
            .add_option(&["-r", "--redis_ip"], Store, "Redis Server IP address");
        ap.refer(&mut period).add_option(
            &["-p", "--period"],
            Store,
            "Retrieval period (default: 1w; max: 99y)",
        );
        ap.parse_args_or_exit();
    }

    // client is a RedisResult enum
    let client = match Client::open(format!("redis://{redis_ip}/")) {
        Ok(c) => c,
        Err(error) => panic!("{error}"),
    };

    let mut conn = match client.get_connection() {
        Ok(conn) => {
            println!("\nConnection to redis server at {redis_ip} successful.\n");
            conn
        }
        Err(error) => panic!("I was not able to establish a connection with redis: {error}"),
    };

    let tickers: Vec<String> = retrieve_tickers(&mut conn);
    get_historical(&mut conn, &tickers, &mut period);
}
