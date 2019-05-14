use chrono::format::ParseResult;
use chrono::offset::FixedOffset;
use chrono::{DateTime, Local, NaiveDateTime};
use mp4parse::MediaContext;
use mp4parse_capi::*;
use regex::Regex;
use rexiv2::Metadata;
use std::path::{Path, PathBuf};
use std::process::Command;
use structopt::StructOpt;

fn main() -> Result<(), std::io::Error> {
    let args = Cli::from_args();
    // let meta = Metadata::new_from_path(&args.src_path).unwrap();
    println!("{:?}", args.src_path);
    println!("{:?}", args.dest_path);
    // println!("{:?}", meta);
    // println!("{:?}", meta.get_tag_string("Exif.Photo.DateTimeOriginal"));

    // let meta2 = Metadata::new_from_path(args.dest_path).unwrap();
    // println!("{:?}", meta.supports_xmp());
    // println!("{:?}", meta.get_xmp_tags());
    // match ffmpeg::format::input(&args.dest_path) {
    //     Ok(context) => {
    //         for (k, v) in context.metadata().iter() {
    //             println!("{}: {}", k, v);
    //         }
    //     }
    //     Err(error) => println!("error: {}", error),
    // }

    // let output = Command::new("exiftool")
    //     .arg("-DateTimeOriginal")
    //     .arg(&args.dest_path)
    //     .output()
    //     .expect("failed to execute process");
    // println!("video: {:?}", output);
    // println!("img: {:?}", get_create_datetime(args.src_path.as_path()));
    // println!("video: {:?}", get_create_datetime(args.dest_path.as_path()));
    //
    visit_dirs(&args.src_path, &args.dest_path)
}

fn visit_dirs(dir: &Path, dpath: &Path) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, dpath)?;
            } else {
                copy(&path, dpath)
            }
        }
    }
    Ok(())
}

fn get_create_datetime(path: &Path) -> String {
    match path.extension() {
        Some(ext) => {
            if ext == "JPG" || ext == "jpg" {
                let meta = Metadata::new_from_path(path).unwrap();
                // println!("{:?}", meta.get_tag_string("Exif.Photo.DateTimeOriginal"));
                let time = meta.get_tag_string("Exif.Photo.DateTimeOriginal").unwrap();

                // println!("{:?}", meta);
                // println!("{:?}", time);

                NaiveDateTime::parse_from_str(
                    &time,
                    // &meta.get_tag_string("Exif.Photo.DateTimeOriginal").unwrap(),
                    "%Y:%m:%d %H:%M:%S",
                )
                .unwrap()
                .to_string()
                .replace(":", ".")
                    + ".jpg"
            } else if ext == "mp4" || ext == "MP4" {
                let output = Command::new("exiftool")
                    .arg("-DateTimeOriginal")
                    .arg(path)
                    .output()
                    .expect("failed to execute process");
                let re = Regex::new(r"\d{4}:\d{2}:\d{2} \d{2}:\d{2}:\d{2}").unwrap();
                let date_string = String::from_utf8(output.stdout).unwrap();
                let mat = re.find(&date_string).unwrap();
                NaiveDateTime::parse_from_str(mat.as_str(), "%Y:%m:%d %H:%M:%S")
                    .unwrap()
                    .to_string()
                    .replace(":", ".")
                    + ".mp4"
            } else {
                panic!("unsupported file")
            }
        }
        None => panic!("failed pares datetime"),
    }
}

fn copy(path: &Path, dropbox_path: &Path) {
    let mut to = format!(
        "{}/{}",
        dropbox_path.to_str().unwrap(),
        get_create_datetime(path)
    );
    if !Path::new(&to).exists() {
        println!("copy: {}", to);
        // std::fs::copy(path, to);
        return;
    }
    // ファイルの中身を確認して同じだったらreturn
    if diff(path.to_str().unwrap(), &to) {
        println!("exists: {}", to);
        return;
    }
    //
    to = to.replace(".jpg", "-0.jpg");
    println!("inc: {}, to {}", path.to_str().unwrap(), to);
    for i in 1.. {
        to = to.replace(
            &format!("-{}.jpg", (i - 1).to_string()),
            &format!("-{}.jpg", (i).to_string()),
        );
        if !Path::new(&to).exists() {
            println!("copy: {}", to);
            // std::fs::copy(path, to);
            return;
        }
        // println!("inc: {}", to);
    }
}

fn diff(path1: &str, path2: &str) -> bool {
    Command::new("diff")
        .arg(path1)
        .arg(path2)
        .status()
        .expect("failed to execute process")
        .success()
}

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    src_path: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    dest_path: std::path::PathBuf,
}
