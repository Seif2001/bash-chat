extern crate steganography;
use steganography::decoder::*;
use steganography::encoder::*;
use steganography::util::*;

// use image::{DynamicImage, RgbaImage, Rgba, ImageBuffer, GenericImageView};
// only the GenericImageView is needed
use image::GenericImageView;
use std::fs::File;
use std::fs::metadata;
use std::io::Read;
// use std::cmp::min;
use std::io::Write;

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