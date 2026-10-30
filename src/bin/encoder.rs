use tar4rg::ansi;
use std::path;

fn batch_create <P: AsRef<path::Path>> (paths: &Vec<P>) -> std::io::Result<Vec<std::fs::File>> {
    let mut files: Vec<std::fs::File> = Vec::new();
    for p in paths {
        files.push(std::fs::File::create(p)?);
    }
    Ok(files)
}

fn file_pack <P: AsRef<path::Path>> (p: &P) -> std::io::Result<Vec<u8>> {
    let mut builder: tar::Builder<Vec<u8>> = tar::Builder::new(Vec::new());
    let path = p.as_ref();
    if path.is_dir() {
        builder.append_dir_all(path.canonicalize()?.file_name().unwrap(), path)?;
    } else {
        builder.append_path_with_name(path, path.file_name().unwrap())?;
    }
    Ok(builder.into_inner()?)
}

fn pack_encode_rg (tar_bytes: Vec<u8>) -> Vec<[u8; 3]> {
    let mut content: Vec<[u8; 3]> = Vec::new();
    // 添加开头符号
    content.push([127, 127, 255]);
    // 循环填补
    for i in (0..tar_bytes.len()).step_by(2) {
        content.push([tar_bytes[i], tar_bytes[i+1], 0]);
    }
    content
}
fn pack_encode_rb (tar_bytes: Vec<u8>) -> Vec<[u8; 3]> {
    let mut content: Vec<[u8; 3]> = Vec::new();
    // 添加开头符号
    content.push([127, 255, 127]);
    // 循环填补
    for i in (0..tar_bytes.len()).step_by(2) {
        content.push([tar_bytes[i], 0, tar_bytes[i+1]]);
    }
    content
}
fn pack_encode_gb (tar_bytes: Vec<u8>) -> Vec<[u8; 3]> {
    let mut content: Vec<[u8; 3]> = Vec::new();
    // 添加开头符号
    content.push([255, 127, 127]);
    // 循环填补
    for i in (0..tar_bytes.len()).step_by(2) {
        content.push([0, tar_bytes[i], tar_bytes[i+1]]);
    }
    content
}

fn png_annotate (content: Vec<[u8; 3]>) -> Vec<[u8; 3]> {
    let mut data: Vec<[u8; 3]> = content;
    let mut size = data.len();
    let side;
    // 计算边长
    if size.isqrt() * size.isqrt() == size {
        side = (size+36).isqrt().div_ceil(8) * 8;
    } else {
        side = ((size+36).isqrt()+1).div_ceil(8) * 8;
    }
    data.resize(side*side, [0_u8; 3]);
    // 长度更新
    size = data.len();
    // 在最后四位放入红绿蓝灰四个颜色，采用灰是为了避开黑白
    data[size-4..].copy_from_slice(&[
        [255_u8, 0, 0], [0, 255, 0], [0, 0, 255], [127, 127, 127]
    ]);
    // 计算写入CRC32C
    let c32c = crc32c::crc32c(&data[..(size - 36)].concat());
    for i in 0..32 {
        data[size - 36 +i] = if (c32c >> i) & 1 == 1 {
            [255, 255, 255]
        } else {
            [0, 0, 0]
        };
    }
    data
}

fn sort_convert(data: Vec<[u8; 3]>) -> Vec<[u8; 3]> {
    let size = data.len();
    let side = size.isqrt();
    let mut png: Vec<[u8; 3]> = vec![[0u8; 3]; size];
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
                png[ pos[0] + pos[1]*side ] = data[i];
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
    png
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut paths: Vec<path::PathBuf> = Vec::new();
    let mut targets: Vec<path::PathBuf> = Vec::new();
    // 收集命令行输入的参数
    let mut args_taken: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if args_taken.len() == 1 {
        println!("{}\n", ansi::red("didn't received path"));
        return Ok(());
    }
    // 读取可能的颜色参数
    let color_encode: fn(Vec<u8>) -> Vec<[u8; 3]>;
    if args_taken[1] == "-rg" {
        color_encode = pack_encode_rg;
        args_taken.swap_remove(1);
    }
    else if args_taken[1] == "-rb" {
        color_encode = pack_encode_rb;
        args_taken.swap_remove(1);
    }
    else if args_taken[1] == "-gb" {
        color_encode = pack_encode_gb;
        args_taken.swap_remove(1);
    } else {color_encode = pack_encode_rg}
    // 检查移除参数后情况
    if args_taken.len() == 1 {
        println!("{}\n", ansi::red("didn't received path"));
        return Ok(());
    }
    // 移除第一个参数
    args_taken.swap_remove(0);

    // 检查输入目录和输出目录
    for p in args_taken {
        let pth = path::Path::new(&p);
        if ! pth.try_exists()? {
            println!("{}", ansi::red(&format!("path {} is not exist", pth.display())));
            return Ok(());
        }
        if pth.with_extension("png").try_exists()? {
            println!("{}", ansi::red(&format!("path {} is occupied", pth.with_extension("png").display())));
            return Ok(());
        }
        paths.push(path::PathBuf::from(&p));
        targets.push(pth.with_extension("png"));
    }
    // 创建目标文件队列
    batch_create(&targets)?;

    // 循环处理各个文件
    for i in 0..targets.len() {
        let tar_bytes = file_pack(&paths[i])?;
        let conctent_bytes = color_encode(tar_bytes);
        let buf_bytes = png_annotate(conctent_bytes);
        let side = buf_bytes.len().isqrt();
        let png_bytes = sort_convert(buf_bytes);
        lodepng::encode24_file(&targets[i], &png_bytes, side, side)?;
    }
    println!("{}", ansi::green("finished"));
    Ok(())
}
