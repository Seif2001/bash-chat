extern crate steganography;
use steganography::decoder::*;
use steganography::encoder::*;
use steganography::util::*;
// use image::{DynamicImage, RgbaImage, Rgba, ImageBuffer, GenericImageView};
// only the generic image view is needed
use image::GenericImageView;
use std::fs::File;
use std::fs::metadata;
use std::io::Read;
use std::io::{self, Write};
use std::path::Path; // Add this import
// use std::cmp::min;
// use std::io::Write;


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

    buffer
}

// pub fn decode_image(path_input: String, output_file: String) -> io::Result<()> {
//     let encoded_image = file_as_image_buffer(path_input);
//     println!("Encoded image dimensions: {}x{}", encoded_image.width(), encoded_image.height());

//     let dec = Decoder::new(encoded_image);
//     let out_buffer = dec.decode_alpha();
//     println!("Decoded buffer size (bytes): {}", out_buffer.len());

//     // Create and write directly to the output file
//     let mut file = File::create(&output_file)?;
//     file.write_all(&out_buffer)?; // Write the buffer directly without referencing the Result

//     println!("Decoding complete. The file has been recovered to {}", output_file);
//     Ok(())
// }

// Assume `file_as_image_buffer` and `Decoder` are from the `steganography` library
// and handle the image loading and decoding process.

pub fn decode_image(path_input: String, output_file: String) -> io::Result<()> {
    // Step 1: Ensure the encoded image file exists
    let encoded_image_path = Path::new(&path_input);
    if !encoded_image_path.exists() {
        println!("Error: Encoded image file '{}' does not exist.", path_input);
        return Err(io::Error::new(io::ErrorKind::NotFound, "Encoded image file not found"));
    }

    // Step 2: Load the encoded image
    let encoded_image = file_as_image_buffer(path_input.clone());
    if encoded_image.width() == 0 || encoded_image.height() == 0 {
        println!("Error: Encoded image appears to be empty or invalid.");
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid encoded image"));
    }
    println!("Encoded image dimensions: {}x{}", encoded_image.width(), encoded_image.height());

    // Step 3: Decode the hidden bytes from the alpha channel
    let dec = Decoder::new(encoded_image);
    let out_buffer = dec.decode_alpha();
    if out_buffer.is_empty() {
        println!("Warning: Decoded buffer is empty, indicating an issue in decoding.");
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Decoded buffer is empty"));
    }
    println!("Decoded buffer size (bytes): {}", out_buffer.len());

    // Step 4: Create and write the decoded data directly to the output file
    let mut file = File::create(&output_file)?;
    file.write_all(&out_buffer)?; // Write the buffer directly
    println!("Decoding complete. The file has been saved to '{}'", output_file);

    Ok(())
}
