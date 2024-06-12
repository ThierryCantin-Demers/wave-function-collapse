use bmp::Image;
use wfc::enums::generate_color;
use wfc::rules::extract_rules;
use wfc::state::generate_image;

pub fn generate_bitmap(w: u32, h: u32) -> Image {
    let mut img = Image::new(w, h);
    for (x, y) in img.coordinates() {
        img.set_pixel(x, y, generate_color(x, y).into());
    }
    img
}

pub fn save_bitmap(img: Image, file_name: &str) {
    let _ = img.save(file_name);
}

pub fn read_bitmap(file_name: &str) -> Image {
    let img = bmp::open(file_name).unwrap();
    println!("read_bitmap: {:?}", img);
    img
}

fn main() {
    let file_name = "imgs/noel.bmp";
    let final_file_name = "imgs/noel_final.bmp";
    let input_img = read_bitmap(file_name);
    let rules = extract_rules(&input_img);
    let res_img = generate_image(16, 16, &rules);
    println!("Generated image: {:?}", res_img);
    if let Some(img) = res_img {
        save_bitmap(img, final_file_name);
    }
}
