use std::thread;
use std::io::Write;
use std::net::TcpStream;

use image::{FilterType, GenericImageView, imageops};
use clap::{App, Arg};

const VERSION: &str = "36c3";

fn main() {
    let matches = App::new("pixelflut")
        .version(VERSION)
        .author("cydhra")
        .about("one network to rule them all, one network to find them, one network to bring them \
         all and in the darkness bind them.")
        .arg(Arg::with_name("server")
            .short("s")
            .long("server")
            .takes_value(true)
            .help("target server address using the format <ip-address>:<port>"))
        .arg(Arg::with_name("image")
            .short("i")
            .long("image")
            .takes_value(true)
            .help("image file to flood"))
        .arg(Arg::with_name("width")
            .short("w")
            .long("width")
            .takes_value(true)
            .help("width of the output image. defaults to input width."))
        .arg(Arg::with_name("height")
            .short("h")
            .long("height")
            .takes_value(true)
            .help("height of the output image. defaults to input height."))
        .arg(Arg::with_name("threads")
            .short("t")
            .long("threads")
            .takes_value(true)
            .help("how many threads to start. defaults to 16."))
        .arg(Arg::with_name("xoff")
            .short("x")
            .long("xoffset")
            .takes_value(true)
            .help("output x offset of the image. defaults to 0."))
        .arg(Arg::with_name("yoff")
            .short("y")
            .long("yoffset")
            .takes_value(true)
            .help("output y offset of the image. defaults to 0."))
        .arg(Arg::with_name("instances")
            .short("n")
            .long("instances")
            .takes_value(true)
            .help("how many instances of the image to draw (each with the specified number of \
            threads). defaults to 1."))
        .get_matches();

    let file_name = match matches.value_of("image") {
        None => {
            eprintln!("no image passed");
            return;
        }
        Some(v) => v,
    };

    let mut address = String::new();
    match matches.value_of("server") {
        None => {
            eprintln!("no target was given");
            return;
        }
        Some(v) => address.push_str(v),
    };

    println!("Pixelflut ({})", VERSION);
    println!("reading image \"{}\"...", file_name);
    let mut img = image::open(file_name).unwrap();
    println!("original image dimensions: {} x {}", img.width(), img.height());

    let width = matches.value_of("width").map(|v| v.parse().unwrap()).unwrap_or_else(|| {
        println!("no custom width was given. using original width");
        img.width()
    });

    let height = matches.value_of("height").map(|v| v.parse().unwrap()).unwrap_or_else(|| {
        println!("no custom height was given. using original height");
        img.height()
    });

    let resized = imageops::resize(&mut img, width, height, FilterType::Triangle);
    println!("resized to: {} x {}", resized.width(), resized.height());

    let thread_count = matches.value_of("threads").map(|v| v.parse().unwrap()).unwrap_or(16u32);
    let instances_count = matches.value_of("instances").map(|v| v.parse().unwrap()).unwrap_or(1);
    let column_width = width / thread_count;
    println!("will divide image into {} columns of width {} and will render each column in {} \
    threads", thread_count, column_width, instances_count);

    let xoff = matches.value_of("xoff").map(|v| v.parse().unwrap()).unwrap_or(0);
    let yoff = matches.value_of("yoff").map(|v| v.parse().unwrap()).unwrap_or(0);
    println!("will offset output image by {}|{}", xoff, yoff);

    let mut threads = vec![];

    for _n in 0..instances_count {
        for thread_index in 0..thread_count {
            let mut str_buffer = String::new();

            for dx in 0..column_width {
                for dy in 0..height {
                    let x = (thread_index * column_width) + dx % width;
                    let y = dy % height;
                    let color = resized.get_pixel(x as u32, y as u32);

                    let pixel = format!("{:02x}{:02x}{:02x}{:02x}", color[0], color[1], color[2], 0xff);
                    str_buffer.push_str(&format!("PX {} {} {}\n", x + xoff, y + yoff, pixel));
                }
            }

            let addr_clone = address.clone();
            threads.push(thread::spawn(move || flood(addr_clone, str_buffer)));
        }
    }

    println!("threads launched. joining...");
    for handle in threads {
        handle.join().unwrap();
        println!("one thread died!")
    }

    println!("all threads died. time to go...");
}

fn flood(address: String, buffer: String) {
    let mut stream = match TcpStream::connect(address) {
        Ok(stream) => stream,
        Err(e) => {
            println!("failed to connect to server:\n{:?}", e);
            return;
        }
    };
    let byte_buffer = buffer.as_bytes();

    loop {
        stream.write(byte_buffer).unwrap();
    }
}