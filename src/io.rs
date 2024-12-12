use bt_hci::data;
use core::fmt::Arguments;
use defmt::info;
use embassy_rp::rtc::{DateTime, DayOfWeek};
use heapless::{String, Vec};

#[allow(dead_code)]

/// Makes it easier to format strings in a single line method
pub fn easy_format<const N: usize>(args: Arguments<'_>) -> String<N> {
    let mut formatted_string: String<N> = String::<N>::new();

    let result = core::fmt::write(&mut formatted_string, args);

    match result {
        Ok(_) => formatted_string,
        Err(_) => {
            panic!("Error formatting the string")
        }
    }
}

pub fn easy_format_str<'a>(
    args: Arguments<'_>,
    buffer: &'a mut [u8],
) -> Result<&'a str, core::fmt::Error> {
    // let mut response_buffer = [0u8; 4096]; // Size the buffer appropriately
    let mut writer = BufWriter::new(buffer);
    let result = core::fmt::write(&mut writer, args);

    match result {
        Ok(_) => {
            let len = writer.len();
            let response_str = core::str::from_utf8(&buffer[..len]).unwrap();
            Ok(response_str)
        }
        Err(_) => {
            panic!("Error formatting the string")
        }
    }
}

/// returns just the time as a string from dates like this 2024-12-10T11:45
pub fn return_str_time(short_datetime: &str) -> &str {
    let split: Vec<&str, 2> = short_datetime.split("T").collect();
    split[1]
}

///Formats dates like this: 2024-12-10
pub fn format_date(date: &str) -> DateTime {
    let split: Vec<&str, 3> = date.split("-").collect();
    let year = split[0].parse::<u16>().unwrap();
    let month = split[1].parse::<u8>().unwrap();
    let day = split[2].parse::<u8>().unwrap();

    DateTime {
        year,
        month,
        day,
        day_of_week: DayOfWeek::Sunday,
        hour: 0,
        minute: 0,
        second: 0,
    }
}

/// Formats datetimes like this: 2024-12-10T11:45
pub fn format_short_datetime(short_datetime: String<16>) -> DateTime {
    let time_date_split = short_datetime.split('T').collect::<Vec<&str, 2>>();
    let date = time_date_split[0];
    let time = time_date_split[1];

    let time_split = time.split(':').collect::<Vec<&str, 2>>();
    let hour = time_split[0].parse::<u8>().unwrap();
    let minute = time_split[1].parse::<u8>().unwrap();

    let date_split = date.split('-').collect::<Vec<&str, 3>>();
    let year = date_split[0].parse::<u16>().unwrap();
    let month = date_split[1].parse::<u8>().unwrap();
    let day = date_split[2].parse::<u8>().unwrap();

    DateTime {
        year,
        month,
        day,
        hour,
        minute,
        //Do not know the second or day of the week. Mostly just used here to have a common return type
        second: 0,
        day_of_week: DayOfWeek::Sunday,
    }
}

/// Formats datetimes like this: 2024-12-10T12:03:59.253687-06:00
pub fn format_long_datetime(datetime: &str, numeric_day_of_the_week: Option<u8>) -> DateTime {
    let datetime = datetime.split('T').collect::<Vec<&str, 2>>();
    //split at -
    let date = datetime[0].split('-').collect::<Vec<&str, 3>>();
    let year = date[0].parse::<u16>().unwrap();
    let month = date[1].parse::<u8>().unwrap();
    let day = date[2].parse::<u8>().unwrap();
    //split at :
    let time = datetime[1].split(':').collect::<Vec<&str, 4>>();
    let hour = time[0].parse::<u8>().unwrap();
    let minute = time[1].parse::<u8>().unwrap();
    //split at .
    let second_split = time[2].split('.').collect::<Vec<&str, 2>>();
    let second = second_split[0].parse::<f64>().unwrap();

    let day_of_the_week = match numeric_day_of_the_week {
        Some(day) => match day {
            0 => DayOfWeek::Sunday,
            1 => DayOfWeek::Monday,
            2 => DayOfWeek::Tuesday,
            3 => DayOfWeek::Wednesday,
            4 => DayOfWeek::Thursday,
            5 => DayOfWeek::Friday,
            6 => DayOfWeek::Saturday,
            _ => DayOfWeek::Sunday,
        },
        //If none just set to sunday because it probably does not matter if it was not passed in
        None => DayOfWeek::Sunday,
    };

    DateTime {
        year: year,
        month: month,
        day: day,
        day_of_week: day_of_the_week,
        hour,
        minute,
        second: second as u8,
    }
}

// A simple wrapper struct to use core::fmt::Write on a [u8] buffer
pub struct BufWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> BufWriter<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        BufWriter { buf, pos: 0 }
    }

    pub fn len(&self) -> usize {
        self.pos
    }
}

impl<'a> core::fmt::Write for BufWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        if self.pos + bytes.len() > self.buf.len() {
            return Err(core::fmt::Error); // Buffer overflow
        }

        self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
        Ok(())
    }
}
