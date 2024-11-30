extern crate steganography;
use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;
use std::io::BufReader;
use image::ImageFormat;
use steganography::decoder::*;
use steganography::encoder::*;
use steganography::util::*;
use show_image::{create_window, ImageInfo, ImageView};
use image::{DynamicImage, FilterType};
use std::fs::{self, File};
use std::io::Error;
use serde::{Serialize, Deserialize};


// use image::{DynamicImage, RgbaImage, Rgba, ImageBuffer, GenericImageView};
// only the GenericImageView is needed
use image::GenericImageView;
use std::fs::metadata;
use std::io::Read;
// use std::cmp::min;
use std::io::Write;

// Struct representing the image request data
#[derive(Serialize, Deserialize, Debug)]
pub struct ImageRequest {
    pub client_username: String,
    pub image_name: String,
    pub is_high: bool,
}

pub fn encode_image(path_input: String, path_output: String ,path_default:String) {
    let payload_bytes = get_file_as_byte_vec(&path_input);
    println!("Payload size (bytes): {}", payload_bytes.len());
    let destination_image = file_as_dynamic_image(path_default);
    println!("Destination image dimensions: {}x{}", destination_image.width(), destination_image.height());
    let enc = Encoder::new(&payload_bytes, destination_image);
    let result = enc.encode_alpha();
    save_image_buffer(result, path_output.clone());
    println!("Encoding complete. The file has been embedded in {}", path_output);
}


fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    println!("Reading file: {}, Size: {} bytes", filename, metadata.len());
    f.read(&mut buffer).expect("buffer overflow");
    // println!("{:?}",buffer);

    buffer
}

pub fn decode_image(path_input: String, output_file: String) {
    // Step 1: Load the encoded image
    let encoded_image = file_as_image_buffer(path_input);
    //let encoded_image = get_file_as_byte_vec()
    
    // Debug: Log image dimensions
    println!("Encoded image dimensions: {}x{}", encoded_image.width(), encoded_image.height());

    let dec = Decoder::new(encoded_image);

    // Step 2: Decode the hidden bytes from the alpha channel
    let out_buffer = dec.decode_alpha();
    
    // Debug: Log the size of the decoded buffer
    println!("Decoded buffer size (bytes): {}", out_buffer.len());

    // Step 3: Write the decoded buffer (extracted file) directly to a new file without filtering
    let mut file = File::create(output_file.clone()).expect("Failed to create output file");
    file.write_all(&out_buffer).expect("Failed to write to file");

    println!("Decoding complete. The file has been recovered to {}", output_file);
}



pub fn display_image(image_path: &str) {
    if let Ok(img) = image::open(image_path) {
        let img = img.to_rgb(); // Convert to RGB format
        let (width, height) = img.dimensions();
        
        let window = create_window("Image Viewer", Default::default()).unwrap();

        let image_info = ImageInfo::rgb8(width, height);
        let image_view = ImageView::new(image_info, &img);
        window.set_image("Image", image_view).unwrap();
    } else {
        println!("Failed to load image.");
    }
}


pub fn resize_image(input_path: &str, output_dir: &str) -> Result<(), Error> {
    // Create the output directory if it doesn't exist
    
    // Open the image file from the given input path
    let file = File::open(input_path)?;

    // Create a BufReader around the file
    let reader = BufReader::new(file);

    // Try to load the image
    let image = match image::load(reader, ImageFormat::JPEG) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Error loading image: {}", e);
            return Err(Error::new(std::io::ErrorKind::Other, e));
        }
    };

    // Resize the image to 100x100
    let resized_img = image.resize(100, 100, FilterType::Lanczos3);

    match resized_img.save(output_dir) {
        Ok(_) => {
            Ok(())
        },
        Err(e) => {
            eprintln!("Failed to save resized image: {}", e);
            Err(e)
        }
    }
}

pub fn append_views(encoded_image_path: String, output_image_path: String, views: u32) {
    let mut f = File::open(&encoded_image_path).expect("No file found");
    let metadata = metadata(&encoded_image_path).expect("Unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("Buffer overflow");

    let views_bytes = views.to_be_bytes();
    buffer.extend_from_slice(&views_bytes);

    let mut output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(output_image_path)
        .expect("Failed to create output file");

    output_file
        .write_all(&buffer)
        .expect("Failed to write to output file");

    // println!("Image and views appended. The new file has been saved to {}", output_image_path);
}

pub fn get_views(encoded_image_path: String) -> std::io::Result<u32> {
    let mut file = File::open(&encoded_image_path)?;
    let metadata = file.metadata()?;
    let mut buffer = vec![0; metadata.len() as usize];
    file.read(&mut buffer)?;
    if buffer.len() < 4 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Insufficient data to extract views"));
    }
    let views_bytes = &buffer[buffer.len() - 4..];
    let views = u32::from_be_bytes(views_bytes.try_into().unwrap());
    let image_data = &buffer[..buffer.len() - 4];
    // let mut output_file = File::create(encoded_image_path)?;
    // output_file.write_all(image_data)?;
    //println!("Image saved without views. Number of views extracted: {}", views);

    Ok(views)
}

// pub fn update_views(encoded_image_path: String) -> std::io::Result<u32> {
//     let mut file = File::open(&encoded_image_path)?;
//     let metadata = file.metadata()?;
//     let mut buffer = vec![0; metadata.len() as usize];
//     file.read(&mut buffer)?;
//     if buffer.len() < 4 {
//         return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Insufficient data to extract views"));
//     }
//     let views_bytes = &buffer[buffer.len() - 4..];
//     let views = u32::from_be_bytes(views_bytes.try_into().unwrap());
//     let image_data = &buffer[..buffer.len() - 4];
//     // let mut output_file = File::create(encoded_image_path)?;
//     // output_file.write_all(image_data)?;
//     //println!("Image saved without views. Number of views extracted: {}", views);

//     Ok(views)
// }
pub fn write_into_json(client_username: String, image_name: String, is_high: bool) -> std::io::Result<()>{
    let image_request = ImageRequest {
        client_username,
        image_name: image_name.clone(),
        is_high,
    };
    // Serialize the struct to a JSON string
    let json_data = serde_json::to_string(&image_request).expect("Failed to serialize data");

    // Specify the file path to write the JSON data
    let file_path = "image_requests_unfinished.json";

    // Create or overwrite the file with the JSON data
    let mut file = File::create(file_path)?;

    // Write the JSON string to the file
    file.write_all(json_data.as_bytes())?;
    println!("Data written to JSON: {}", json_data);
    Ok(())
}

// Function to read and deserialize the JSON file
pub fn read_image_requests(file_path: &str) -> std::io::Result<Vec<ImageRequest>> {
    let mut file = File::open(file_path)?;
    
    // Check if the file is empty
    let metadata = file.metadata()?;
    if metadata.len() == 0 {
        return Ok(Vec::new()); // Return an empty vector if the file is empty
    }

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    // Try to deserialize the contents
    match serde_json::from_str::<Vec<ImageRequest>>(&contents) {
        // If it's a valid array of requests, return it
        Ok(requests) => Ok(requests),
        // If the JSON is not an array, check if it's an object (map)
        Err(_) => {
            // Try to deserialize as a single map entry or handle specific error
            match serde_json::from_str::<ImageRequest>(&contents) {
                Ok(request) => Ok(vec![request]), // Return a single item as a vec
                Err(e) => {
                    eprintln!("Error deserializing JSON: {}", e);
                    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to parse JSON"))
                }
            }
        }
    }
}

// Function to clear the file (optional, if you want to clear the file after processing)
pub fn clear_file(file_path: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;  // This will truncate the file to 0 size
    file.write_all(b"")?;  // Optionally write an empty byte slice to clear the file
    Ok(())
}
