#[derive(PartialEq, Clone)]
pub struct Arguments {
    // pub no_tui: bool,
    // pub location: String,
    pub help: bool,
    // pub forecast: i32,
}

pub fn parse() -> Arguments {
    let mut it = std::env::args().skip(1); // skip program name
    // let mut no_tui = false;
    // let mut location = String::from("Stockholm");
    let mut help = false;
    // let mut forecast = DEF_FORECAST;
    while let Some(arg) = it.next() {
        match arg.as_str() {
            // "-l" => {
            //     location = it
            //         .next()
            //         .expect("No location was given after the \"-l\" flag.");
            // }
            "help" | "-h" | "h" => {
                help = true;
            }
            // "-t" => {
            //     no_tui = true;
            // }
            // "-f" => {
            //     // use next if some and parse to i32, else default
            //     forecast = it
            //         .next()
            //         .as_deref()
            //         .unwrap_or(format!("{}", DEF_FORECAST).as_str())
            //         .parse::<i32>()
            //         .unwrap_or(DEF_FORECAST);
            // }
            _ => {}
        }
    }

    // if forecast > MAX_FORECAST {
    //     forecast = MAX_FORECAST
    // }

    Arguments {
        // no_tui,
        // location,
        help,
        // forecast,
    }
}
