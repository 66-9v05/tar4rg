use tar4rg::ansi;
use std::path;

fn png_decode_rg(png: Vec<[u8; 3]>) -> Vec<u8> {
    let mut tar_bytes: Vec<u8> = Vec::new();
    for pixel in png[1..png.len()-36].iter() {
        tar_bytes.push(pixel[0]);
        tar_bytes.push(pixel[1]);
    }
    tar_bytes
}
fn png_decode_rb(png: Vec<[u8; 3]>) -> Vec<u8> {
    let mut tar_bytes: Vec<u8> = Vec::new();
    for pixel in png[1..png.len()-36].iter() {
        tar_bytes.push(pixel[0]);
        tar_bytes.push(pixel[2]);
    }
    tar_bytes
}
fn png_decode_gb(png: Vec<[u8; 3]>) -> Vec<u8> {
    let mut tar_bytes: Vec<u8> = Vec::new();
    for pixel in png[1..png.len()-36].iter() {
        tar_bytes.push(pixel[1]);
        tar_bytes.push(pixel[2]);
    }
    tar_bytes
}

fn png_check(png: &Vec<[u8; 3]>) -> Option<fn(Vec<[u8; 3]>) -> Vec<u8>> {
    let size = png.len();
    let side = size.isqrt();
    let decoder: fn(Vec<[u8; 3]>) -> Vec<u8>;
    // 检查长度是否合理，注意并不检查是否为严格正方形，因为没有必要
    if side * side != size {return None;}
    if side % 8 != 0 {return None;}
    // 检查开头标记
    decoder = match png[0] {
        [127, 127, 255] => png_decode_rg,
        [127, 255, 127] => png_decode_rb,
        [255, 127, 127] => png_decode_gb,
        _ => return None,
    };
    // 检查中央四格，pos = x + y * width
    let red_pos = (side/2 -1) + (side/2 -1) * side;
    if png[red_pos] != [255, 0, 0] {return None;}
    if png[red_pos+1] != [0, 255, 0] {return None;}
    if png[red_pos+1+side] != [0, 0, 255] {return None;}
    if png[red_pos+side] != [127, 127, 127] {return None;}

    Some(decoder)
}

fn crc32c_check(content: &Vec<[u8; 3]>) -> bool {
    let mut c32c: u32 = 0;
    let size = content.len();
    for i in 0..32 {
        if content[size - 36 + i] == [255, 255, 255] {
            c32c |= 1 << i;
        }
    }
    if crc32c::crc32c(&content[..(size - 36)].concat()) != c32c {
        return false;
    }
    true
}

fn sort_revert(png: Vec<[u8; 3]>) -> Vec<[u8; 3]> {
    let size = png.len();
    let side = size.isqrt();
    let mut data: Vec<[u8; 3]> = vec![[0u8; 3]; size];
    // 写入点
    let mut pos: [usize; 2] = [0, 0];
    // 读取点
    let mut i: usize = 0;
    // 螺旋图像的层数递减修正
    let mut lyr: usize = 1;
    // 索引从零开始，所以舍弃等于条件
    while i < size {
        // 向四个方向的加值，通过简单的枚举来实现
        for [x, y] in [[1_i32, 0], [0_i32, 1], [-1_i32, 0], [0_i32, -1]] {
            for _ in 0..(side-lyr) {
                // 将读取点的数据写入写入点
                data[i] = png[ pos[0] + pos[1]*side ];
                // 先原位置绘制再移动位置
                // eprintln!("info: {}, {}, {}, {side}, direction=[{x},{y}]", i, pos[0], pos[1]);
                pos[0] = (pos[0] as i32 + x) as usize;
                pos[1] = (pos[1] as i32 + y) as usize;
                i += 1;
            }
        }
        // 因为每层会减少两个像素长度
        lyr += 2;
        // 因为每次一圈的螺旋会回到起点所以需要向右下角移动
        pos[0] += 1;
        pos[1] += 1;
    };
    data
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 收集命令行输入的参数
    let mut args_taken: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if args_taken.len() == 1 {
        println!("{}", ansi::red("didn't received path"));
        return Ok(());
    }
    args_taken.swap_remove(0);
    // 检查输入
    for p in &args_taken {
        let pth = path::Path::new(p);
        if ! pth.try_exists()? {
            println!("{}", ansi::red(&format!("path {} is not exist", pth.display())));
            return Ok(());
        }
        match pth.extension() {
            Some(s) => {
                if s != "png" {
                    println!("{}", ansi::red(&format!("file {} is not a png file", pth.display())));
                    return Ok(());
                }
            },
            _ => {
                println!("{}", ansi::red(&format!("file {} is not a png file", pth.display())));
                return Ok(());
            }
        }
    }
    // 循环处理
    for p in &args_taken {
        let pth = path::Path::new(p);
        // 读取图片并转换为Vec<[u8; 3]>
        let png_pixels: Vec<[u8; 3]> = lodepng::decode24_file(pth)?
            .buffer.iter().map(|p| [p.r, p.g, p.b]).collect();
        // 检查是否可读并识别颜色模式返回对应的解码器
        let png_decode = match png_check(&png_pixels) {
            Some(func) => func,
            None => {
                println!("{}", ansi::red(&format!("file {} is an unreadable png", pth.display())));
                return Ok(());
            }
        };
        let content = sort_revert(png_pixels);
        if ! crc32c_check(&content) {
            println!("{}", ansi::red(&format!("file {} has incorrect crc32c code", pth.display())));
            return Ok(());
        }
        let tar_bytes = png_decode(content);
        let mut file_archive = tar::Archive::new(tar_bytes.as_slice());
        let mut file_entry = file_archive.entries()?.next().unwrap()?;
        let target = pth.canonicalize()?.parent().unwrap().join(file_entry.path()?.file_name().unwrap());
        if target.try_exists()? {
            println!("{}", ansi::red(&format!("path {} is occupied", target.display())));
            return Ok(());
        }
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut file_entry, &mut buf)?;
        std::fs::write(target, &buf)?;
    }
    println!("{}", ansi::green("finished"));
    Ok(())
}
