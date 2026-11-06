use raylib::prelude::*;
use raylib::core::texture::RaylibTexture2D;

pub struct Framebuffer {
    pub height: u32,
    pub width: u32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
    texture: Option<Texture2D>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let background_color = Color::BLACK;
        let color_buffer = Image::gen_image_color(width.try_into().unwrap(), height.try_into().unwrap(), background_color);
        Framebuffer { 
            height, 
            width, 
            color_buffer, 
            background_color, 
            current_color: Color::WHITE,
            texture: None,
        }
    }

    pub fn clear(&mut self){
        self.color_buffer = Image::gen_image_color(self.width.try_into().unwrap(), self.height.try_into().unwrap(), self.background_color);
    }

    pub fn set_pixel(&mut self, x: u32, y: u32){

        if x < self.width && y < self.height {
            self.color_buffer.draw_pixel(x as i32, y as i32, self.current_color);
        }
    }

    pub fn point(&mut self, x: i32, y: i32, color: Vector3) {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            let to_byte = |component: f32| (component.clamp(0.0, 1.0) * 255.0) as u8;
            self.color_buffer.draw_pixel(
                x,
                y,
                Color::new(to_byte(color.x), to_byte(color.y), to_byte(color.z), 255),
            );
        }
    }

    pub fn set_background_color(&mut self, color: Vector3){
        let to_byte = |component: f32| (component.clamp(0.0, 1.0) * 255.0) as u8;
        self.background_color = Color::new(to_byte(color.x), to_byte(color.y), to_byte(color.z), 255);
    }

    pub fn set_current_color(&mut self, color: Color){
        self.current_color = color;
    }

    pub fn init_texture(&mut self, window: &mut RaylibHandle, thread: &RaylibThread) {
        self.texture = Some(window.load_texture_from_image(thread, &self.color_buffer)
            .expect("No se pudo crear la textura del framebuffer"));
    }

    pub fn swap_buffers(&mut self, window: &mut RaylibHandle, thread: &RaylibThread) {
        let pixels = self.color_buffer.get_image_data_u8(false);
        if let Some(texture) = &mut self.texture {
            texture.update_texture(&pixels).expect("No se pudo actualizar la textura");
            let mut drawing = window.begin_drawing(thread);
            drawing.clear_background(Color::BLACK);
            drawing.draw_texture(texture, 0, 0, Color::WHITE);
        }
    }

    pub fn render_to_file(&self, file_path: &str){
        self.color_buffer.export_image(file_path);
    }

pub fn render_to_bmp(&self, file_path: &str) {
    use std::fs::File;
    use std::io::Write;

    let width = self.width as usize;
    let height = self.height as usize;

    // Cada fila debe ser múltiplo de 4 bytes
    let row_size = (width * 3 + 3) & !3;
    let pixel_array_size = row_size * height;
    let file_size = 54 + pixel_array_size;

    let mut file = File::create(file_path).unwrap();

    // ===== BMP HEADER =====
    file.write_all(b"BM").unwrap();
    file.write_all(&(file_size as u32).to_le_bytes()).unwrap();
    file.write_all(&[0u8; 4]).unwrap();
    file.write_all(&(54u32).to_le_bytes()).unwrap();

    // ===== DIB HEADER =====
    file.write_all(&(40u32).to_le_bytes()).unwrap();
    file.write_all(&(width as i32).to_le_bytes()).unwrap();
    file.write_all(&(height as i32).to_le_bytes()).unwrap();
    file.write_all(&(1u16).to_le_bytes()).unwrap();
    file.write_all(&(24u16).to_le_bytes()).unwrap();
    file.write_all(&(0u32).to_le_bytes()).unwrap();
    file.write_all(&(pixel_array_size as u32).to_le_bytes()).unwrap();
    file.write_all(&(0i32).to_le_bytes()).unwrap();
    file.write_all(&(0i32).to_le_bytes()).unwrap();
    file.write_all(&(0u32).to_le_bytes()).unwrap();
    file.write_all(&(0u32).to_le_bytes()).unwrap();

    // Padding por fila
    let padding = row_size - width * 3;
    let padding_bytes = vec![0u8; padding];

    // BMP guarda las filas de abajo hacia arriba
    for y in (0..height).rev() {
        for x in 0..width {

            let color = self.color_buffer.get_color(x as i32, y as i32);

            // BMP usa BGR
            file.write_all(&[
                color.b,
                color.g,
                color.r,
            ]).unwrap();
        }

        file.write_all(&padding_bytes).unwrap();
    }
}
}
