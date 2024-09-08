use rand::seq::{IndexedRandom, SliceRandom};
use rand::Rng;
use std::fs::File;
use std::io::Write;


/**
    * Generates random temperature data for cities from a list of cities in a CSV file.
    * A maximum of 10000 cities are selected.
    *
    * Usage: ./generator <input csv file> <output csv file> <number of rows>
    *
    * Input CSV file must contain the city name in the first column.
    * Output CSV file has two columns: city name and temperature.
    * The temperature is a random one decimal value between -60.0 and 60.0.
    */
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        println!("Usage: ./generator <input csv file> <output csv file> <number of rows>");
        std::process::exit(1);
    }
    let input_file = &args[1];
    let output_file = &args[2];
    let num_rows = match args[3].parse::<usize>() {
        Ok(r) => r,
        Err(e) => panic!("{}", e.to_string()),
    };

    let mut cities: Vec<Vec<u8>> = Vec::new();
    match csv::Reader::from_path(input_file) {
        Ok(mut rdr) => {
            for result in rdr.records() {
                match result {
                    Ok(record) => {
                        if record.len() == 0 {
                            continue;
                        }
                        let mut bytes = record[0].as_bytes().to_vec();
                        bytes.push(b';');
                        cities.push(bytes);
                    },
                    Err(e) => panic!("{}", e),
                }
            }
        },
        Err(e) => panic!("{}", e),
    };

    cities.shuffle(&mut rand::thread_rng());

    // Limit to 10000 cities
    if cities.len() > 10000 {
        cities.truncate(10000);
    }

    let max_city_len = cities.iter().map(|v| v.len()).max().unwrap_or_else(|| 0);
    let max_record_len = max_city_len + "-99.9\n".len();
    let max_file_size = max_record_len * num_rows;

    println!("Generating {} rows of data, with a potential file size of {} bytes", num_rows, max_file_size);

    let mut file = File::create(output_file).unwrap_or_else(|e| panic!("{}", e));
    file.set_len(max_file_size as u64).unwrap_or_else(|e| panic!("{}", e));

    let mut written = 0usize;

    for _ in 0..num_rows {
        let city = cities.choose(&mut rand::thread_rng()).unwrap_or_else(|| panic!("No city found"));
        let temp = rand::thread_rng().gen_range(-600..600);
        match write_measurement(&mut file, city, temp) {
            Ok(w) => written += w,
            Err(e) => panic!("{}", e),
        }
    }

    file.flush().unwrap_or_else(|e| panic!("{}", e));
    file.set_len(written as u64).unwrap_or_else(|e| panic!("{}", e));

    println!("Wrote {} bytes to {}", written, output_file);
}

fn write_measurement(file: &mut File, city: &Vec<u8>, mut measurement: i32) -> Result<usize, String> {
    let mut written = 0usize;

    match file.write(city) {
        Ok(w) => written += w,
        Err(e) => return Err(e.to_string()),
    }

    let mut buf: [u8; 6] = [0; 6];
    let mut buf_pos = 0;
    if measurement < 0 {
        buf[buf_pos] = b'-';
        buf_pos = 1;
        measurement = -measurement;
    }
    let mut scale = 100;
    if measurement < 100 {
        scale = 10;
    }
    while scale > 1 {
        buf[buf_pos] = b'0' + (measurement / scale) as u8;
        measurement %= scale;
        scale /= 10;
        buf_pos += 1;
    }
    buf[buf_pos] = b'.';
    buf[buf_pos + 1] = b'0' + measurement as u8;
    buf[buf_pos + 2] = b'\n';
    buf_pos += 3;

    match file.write(&buf[0..buf_pos]) {
        Ok(w) => written += w,
        Err(e) => return Err(e.to_string()),
    }

    Ok(written)
}