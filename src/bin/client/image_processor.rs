extern crate steganography;
use std::fs::OpenOptions;
use steganography::decoder::*;
use steganography::encoder::*;
use steganography::util::*;
use show_image::{create_window, ImageInfo, ImageView};

// use image::{DynamicImage, RgbaImage, Rgba, ImageBuffer, GenericImageView};
// only the GenericImageView is needed
use image::GenericImageView;
use std::fs::File;
use std::fs::metadata;
use std::io::Read;
use std::io::Result;

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

pub fn create_small_image(path_input: String,output_file: String){
    let img = image::open(&path_input).unwrap();
    let img = img.thumbnail(100, 100);
    img.save(output_file).unwrap();
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

pub fn get_views(encoded_image_path: String) -> Result<u32> {
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
    let mut output_file = File::create(encoded_image_path)?;
    output_file.write_all(image_data)?;
    println!("Image saved without views. Number of views extracted: {}", views);

    Ok(views)
}
