extern crate image;
extern crate math;
use std::fmt;
use image::{GenericImage, ImageBuffer,RGB,GenericImageView, DynamicImage};
use image::imageops;
use std::cmp::{min, max};

#[derive(Debug, Copy, Clone)]
enum Pix{
    R,G,B
}


struct Histogram{
    r_dist : Vec<f32>,
    g_dist : Vec<f32>,
    b_dist : Vec<f32>
}
impl fmt::Debug for Histogram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let L = 256;
        write!(f,"\n----------------- RED -----------------\n");
        for i in 0..L{
            if self.r_dist[i] > 0.0{
                write!(f,"{} => {}\n", i, self.r_dist[i]);
            }
        }
        write!(f,"\n----------------- GREEN -----------------\n");
        for i in 0..L{
            if self.g_dist[i] > 0.0{
                write!(f,"{} => {}\n", i, self.g_dist[i]);
            }
        }
        write!(f,"\n----------------- BLUE -----------------\n");
        for i in 0..L{
            if self.b_dist[i] > 0.0{
                write!(f,"{} => {}\n", i, self.b_dist[i]);
            }
        }
        write!(f,"")
    }
}

impl Histogram{
    fn probability(&self,pix_val: u8)->(f32,f32,f32){
        (self.r_dist[pix_val as usize],self.g_dist[pix_val as usize],self.b_dist[pix_val as usize])
    }
    fn probability_of(&self,p : Pix, pix_val : u8)->f32{
        match p{
            R => self.r_dist[pix_val as usize], 
            G => self.g_dist[pix_val as usize], 
            B => self.b_dist[pix_val as usize], 
        }
    }
    fn new(img : & DynamicImage)->Self{
        let (width, height) = img.dimensions();
        let L = 256;
        let mut r_dist = vec![0f32;L];
        let mut g_dist = vec![0f32;L];
        let mut b_dist = vec![0f32;L];
        let sum : f32 = (width * height) as f32;
        for i in 0..width{
            for j in 0..height{
                let r_p = img.get_pixel(i,j).data[0];
                r_dist[r_p as usize] += 1.0;
                let g_p = img.get_pixel(i,j).data[1];
                g_dist[g_p as usize] += 1.0;
                let b_p = img.get_pixel(i,j).data[2];
                b_dist[b_p as usize] += 1.0;
            }
        }
        let mut sum_distros = 0.0;
        for i in 0..L{
            if r_dist[i] >= 1.0{
                r_dist[i] = r_dist[i] / sum;
                sum_distros += r_dist[i];
            }
            if g_dist[i] >= 1.0{
                g_dist[i] = g_dist[i] / sum;
            }
            if b_dist[i] >= 1.0{
                b_dist[i] = b_dist[i] / sum;
            }   
        }
        Histogram{
            r_dist: r_dist,
            g_dist: g_dist,
            b_dist: b_dist
        }
    }
}


fn T(r : u8, p_r : f32, p_0: f32)->f32{
    let L = 256.0;
    (L-1.0) * (p_r)
}
fn t_pixel(rgb: (u8,u8,u8) , p_r_0 : (f32,f32), p_g_0 : (f32,f32) ,p_b_0 : (f32,f32))->(u8,u8,u8){
    let tr = T(rgb.0,p_r_0.0,p_r_0.1) as u8;
    let tg = T(rgb.1,p_g_0.0,p_g_0.1) as u8;
    let tb = T(rgb.2,p_b_0.0,p_b_0.1) as u8;
    (tr,tg,tb)
}
fn transform_pixel(original_pixel : u8, colorType : Pix ,histogram :& Histogram)-> u8{
    let L = 256.0; 
    let new_pixel = 0; 
    let mut distros_sum = 0.0;
    let mut up_to = original_pixel as u32 +1;
    for i in 0..up_to{
        distros_sum += histogram.probability_of(colorType, i as u8);
    }
    ((L-1.0) * distros_sum) as u8
}
/// the real deal 
/// does make image form very bright to darker but in a nice wayu and works with the integral
fn transform_from_his(img : &DynamicImage, hist : &Histogram)->DynamicImage{
    let (w,h) = img.dimensions();
    let mut new_img = DynamicImage::new_rgba8(w, h);
    let pixel = img.get_pixel(w-1, h-1);
    for i in 0..w{
        for j in 0..h{
            let pixel = img.get_pixel(i, j);
            let mut r = pixel.data[0];
            let mut g = pixel.data[1];
            let mut b = pixel.data[2];
            r = transform_pixel(r, Pix::R, hist);
            g = transform_pixel(g, Pix::G, hist);
            b = transform_pixel(b, Pix::B, hist);
            let transformed_pixel = image::Rgba([r,g,b,pixel.data[3]]);
            new_img.put_pixel(i,j, transformed_pixel);
        }
    }
    new_img
}


#[cfg(test)]
mod tests {
    use super::*;
    fn equals(img1 : &DynamicImage, img2 :&  DynamicImage)->bool{
        let (w1,h1) = img1.dimensions();
        let (w2,h2) = img2.dimensions();
        // compare image dimensions 
        if w1 != w2 || h1 != h2 {
            return false;
        }
        for i in 0..w1{
            for j in 0..h1{
                let p1 = img1.get_pixel(i, j);
                let r1 = p1.data[0];
                let g1 = p1.data[1];
                let b1 = p1.data[2];
                let a1 = p1.data[3];
                let p2 = img2.get_pixel(i, j);
                let r2 = p1.data[0];
                let g2 = p1.data[1];
                let b2 = p1.data[2];
                let a2 = p1.data[3];
                if r1 != r2 || g1 != g2 || b1 != b2 || a1 != a2{
                    return false;
                }
            }
        }
        true
    }
    #[test]
    fn perform_histogram_equalization() {
        let path = "./test/bright_miami.jpg";
        let img = image::open(path).unwrap();
        let histogram = Histogram::new(&img);
        let new_image = transform_from_his(&img, &histogram);
        // verify 
        let test_img = image::open("./test/normalized_miami.jpg").unwrap();
        assert!(equals(&test_img, &new_image),"image not equal");
    }
}