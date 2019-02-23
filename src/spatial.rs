use image::{GenericImage,GenericImageView, DynamicImage};
/// Represent a quanternion with (x,y) and len of with and height 
#[derive(Debug,Clone,Copy,Eq, Hash)]
pub struct Quanternion{
    /// `x_w` is the index representing the width column in the image (x,y)
    /// `y_h` is the index representing the height column in the image (x,y)
    /// w is the with length of the square from the starting point (x,y)
    /// h is the height length of the square from the starting point (x,y)
    pub x_w : u32,
    pub y_h : u32, 
    pub w : u32, 
    pub h : u32,
}
impl Quanternion{
    fn new(point : (u32,u32),len_w : u32, len_h : u32)->Self{
        Quanternion {
            x_w : point.0,
            y_h : point.1, 
            w : len_w, 
            h : len_h,
        }
    }
    /// get the edge width of the square 
    fn edge_w(&self)->u32{
        self.x_w + self.w
    }
    /// get the edge height of the square 
    fn edge_h(&self)->u32{
        self.y_h + self.h
    }
}
impl PartialEq for Quanternion {
    fn eq(&self, other: &Quanternion) -> bool {
        self.x_w == other.x_w && 
        self.y_h == other.y_h && 
        self.w == other.w && 
        self.h == other.h
    }
}
/// Color Enum to be used to drawing 
#[derive(Debug,Clone,Copy)]
pub enum Color{
    Red,
    Green,
    Blue,
    Orange,
    Purple, 
}
impl Color{
    ///```
    /// use image_toolbox::spatial::{Color}; 
    /// 
    /// let c = Color::Red; 
    /// println!("R={},G={},B={}", c.val().0,c.val().1,c.val().2);
    ///```
    pub fn val(&self)->(u8,u8,u8){
        match *self{
            Color::Red => (255,0,0),
            Color::Green => (0,255,0),
            Color::Blue => (51,0,255),
            Color::Orange => (255,204,0),
            Color::Purple => (255,51,255),
        }
    }
}

fn draw_square_q(img : &mut DynamicImage, c : Color, q : Quanternion)->bool{
    let f = draw_square(img, c, (q.x_w,q.y_h), (q.w,q.h));
    f
}

fn draw_square(img: &mut DynamicImage, color : Color,x_y : (u32,u32) , w_h : (u32,u32))->bool{
    let (w,h) = img.dimensions();
    // check edges 
    if x_y.0 + w_h.0 > w || x_y.1 + w_h.1 > h{
        return false;
    }
    // draw both horiozontal lines
    let start = x_y.0;
    let end = x_y.0 + w_h.0;
    for i in start..end{
        // top line 
        paint(img, color, (i,x_y.1));
        // bottom line 
        paint(img, color, (i,std::cmp::min(x_y.1 + w_h.1, h-1)));
    }
    // draw both vertical lines 
    let start = x_y.1; 
    let end = w_h.1 + x_y.1;
    for j in start..end{
        // left line 
        paint(img, color, (x_y.0,j));
        // right line 
        paint(img, color, (std::cmp::min(x_y.0+w_h.0,w-1),j));
    }
    true
}

// paint 1 pixel with some color 
fn paint(img : &mut DynamicImage, c : Color, point : (u32,u32)){
    let (r,g,b) = c.val();
    let alpha = img.get_pixel(point.0,point.1).data[3];
    let transformed_pixel = image::Rgba([r,g,b,alpha]);
    img.put_pixel(point.0,point.1, transformed_pixel);
}
// given an image and a square len get all the quaternions sliced with that square that are valid 
fn img_to_squares(img : &DynamicImage, square_size : usize)->Result<Vec<Quanternion>,()>{
    let (w,h) = img.dimensions();
    // validate square size isn't bigger than the image itself 
    if w <= square_size as u32 || h <= square_size as u32{
        return Err(());
    }
    let mut squares : Vec<Quanternion> = Vec::new();
    // calculate squares 
    let s = square_size as u32;
    for i in (0..w).step_by(square_size){
        for j in (0..h).step_by(square_size){
            if is_valid_window(img, (i,j),square_size as u32) {
                squares.push(Quanternion::new((i,j),s, s));
            }
        }
    }
    Ok(squares)
}
// check edges, given a point (x,y) and length of the desired window square check if edges ok in matrix
fn is_valid_window(img : &DynamicImage, point : (u32,u32), len : u32 )->bool{
    let (w,h) = img.dimensions();
    let (x,y) = point;
    return x+len <= w && y+len <= h;
}

/// http://homepages.inf.ed.ac.uk/rbf/CVonline/LOCAL_COPIES/VELDHUIZEN/node18.html
pub fn psnr(mse : f64, S : f64)->f64{
    let c = mse / S.powf(2.0);
    c.log10() * (-10.0)
}
fn mse(est_val : f64 , true_val : f64)->f64{
    let res : f64  = est_val - true_val;
    // res.powf(2.0)
    res.abs()
}
fn mse_vec(est_points : & Vec<u32>,true_points : & Vec<u32>)->Result<f64,()>{
    if est_points.len() - true_points.len() != 0{
        return Err(())
    }
    let size = est_points.len();
    let mut sum : f64 = 0.0;
    for i in 0..size{
        sum += mse(est_points[i] as f64, true_points[i] as f64);
    }
    Ok(sum / size as f64)
}

// treat q1 as the source of truth and q2 es estimators 
pub fn is_equal_mse(img : & DynamicImage, q1 : Quanternion, q2 : Quanternion, err_limit : f64)->Result<bool,()>{
    let rgb_points_q1 = im_to_vec(img, q1);
    let rgb_points_q2 = im_to_vec(img, q2);
    let r_mse = mse_vec(&rgb_points_q2.0, &rgb_points_q1.0)?;
    let g_mse = mse_vec(&rgb_points_q2.1, &rgb_points_q1.1)?;
    let b_mse = mse_vec(&rgb_points_q2.2, &rgb_points_q1.2)?;
    let delta = (r_mse + g_mse + b_mse) / 3.0;
    Ok(delta <= err_limit)
}
// given an img and source square find all the squares that their mse is different 
pub fn find_non_equals_mse(img : &DynamicImage, source : Quanternion , others : Vec<Quanternion>, err_limit : f64)->Vec<Quanternion>{
    let mut result = Vec::new();
    for q in others{
        if !is_equal_mse(img, source, q, err_limit).unwrap(){
            result.push(q);
        }
    }
    result
}
pub fn im_to_vec(img : &DynamicImage, q : Quanternion)->(Vec<u32>,Vec<u32>,Vec<u32>){
    let (w,h) = img.dimensions();
    let mut real_reds = Vec::new();
    let mut real_greens = Vec::new();
    let mut real_blues = Vec::new();
    for i in q.x_w..q.edge_w(){
        for j in q.y_h..q.edge_h(){
            let p = img.get_pixel(i, j);
            real_reds.push(p.data[0] as u32);
            real_greens.push(p.data[1] as u32);
            real_blues.push(p.data[2] as u32);
        }
    }
    (real_reds,real_greens,real_blues)
}

/// draw a vertical line from top to bottom given color, start_point(width,height) and length of line.
/// ```
/// use image_toolbox::{load_img};
/// use image_toolbox::spatial::{draw_vertical_line, Color};
/// 
/// let mut img = load_img("./test/empty_img.jpg").unwrap();
/// let line_length = 150; 
/// let start_point = (50,50);
/// draw_vertical_line(&mut img, Color::Red, start_point, line_length).unwrap();
/// ```
pub fn draw_vertical_line(img : &mut DynamicImage, c : Color, start_point : (u32,u32), length : u32)->Result<(),()>{
    let (x,y) = start_point;
    let (w,h) = img.dimensions();
    // check bounds 
    if x >= w || y + length >= h {
        return Err(());
    }
    for i in 0..length{
        paint(img, c, (x,y+i));
    }
    Ok(())
}
/// draw a hotizontal line from left to right given color, start_point(width,height) and length of line.
/// see draw_vertical_line documentation for usage example.
pub fn draw_horizontal_line(img : &mut DynamicImage, c : Color, start_point : (u32,u32), length : u32)->Result<(),()>{
    let (x,y) = start_point;
    let (w,h) = img.dimensions();
    // check bounds 
    if y >= h || x + length >= w {
        return Err(());
    }
    for i in 0..length{
        paint(img, c, (x + i , y));
    }
    Ok(())
}

fn main(){
    // let path = "./noise_salt_n_pepper.png";
    // let path = "./diff_miami.jpg._500.jpg";
    let path = "./coolimg.jpg";
    // let path = "./me_500.jpg";
    // let mut img = load(path).expect("error loading image");
    // let quaternions = img_to_squares(&img, 50).unwrap();
    // let source = quaternions[0];
    // for q in quaternions {
    //     if is_equal_mse(&img, source, q,60.0).unwrap() {
    //         let mut color = Color::Green;
    //         if q == source {
    //             color = Color::Red;
    //         }
    //         draw_square_q(&mut img, color , q);
    //     }
    // }
    // img.save("./squares.jpg").unwrap();
}
// fn m22ain() {
//     let path = "./square2.jpg";
//     let mut img = load(path).expect("error loading image");
//     println!("w = {}, h ={} ", img.dimensions().0, img.dimensions().1);
//     match draw_square(&mut img,Color::Blue,(30,30), (200,350)){
//         true => img.save("./square2.jpg").unwrap(),
//         false =>println!("error drawing triangle!")
//     }
//     // filter3x3(&img);x`
//     // println!("{:?}",histogram);
//     // let new_image = inhense(&img);
//     // new_image.save("./transformed_integral_probability.jpg").unwrap();
// }
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
    fn draw_vert_line(){
        let path = "./test/empty_img.jpg";
        let mut img = image::open(path).unwrap();
        let line = 499;
        let start = (250,0);
        draw_vertical_line(&mut img, Color::Red ,start, line).unwrap();
        // verify
        let test_img = image::open("./test/vertical_line.jpg").unwrap();
        assert!(equals(&test_img, &img),"image not equal");
    }
    #[test]
    fn draw_horiz_line(){
        let path = "./test/empty_img.jpg";
        let mut img = image::open(path).unwrap();
        let line = 499;
        let start = (0,250);
        draw_horizontal_line(&mut img, Color::Red ,start, line).unwrap();
        // verify
        let test_img = image::open("./test/horizontal_line.jpg").unwrap();
        assert!(equals(&test_img, &img),"image not equal");
    }
    #[test]
    fn draw_square(){

    }
    fn draw_squares(){

    }
    fn draw_non_local_means(){
    }
}