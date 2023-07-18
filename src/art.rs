use image::imageops::FilterType;
use image::*;

use crate::matrix::*;
use crate::sortfns::*;

type ImgGrid = Matrix<(usize, usize)>;

pub fn pixel_map_row(img: &DynamicImage, f: SortFn, grid: Option<ImgGrid>) -> ImgGrid {
    let mut px_map = match grid {
        Some(g) => g,
        None => Matrix::generate(img.width() as usize, img.height() as usize, |x, y| (x, y)),
    };
    for y in 0..px_map.height {
        let mut row = px_map[y].to_vec();
        row.sort_by_key(|x| f(img.get_pixel(x.0 as u32, x.1 as u32)));
        for (i, e) in px_map[y].iter_mut().enumerate() {
            *e = row[i];
        }
    }
    px_map
}

pub fn pixel_map_column(img: &DynamicImage, f: SortFn, grid: Option<ImgGrid>) -> ImgGrid {
    let mut px_map = match grid {
        Some(g) => g,
        None => Matrix::generate(img.width() as usize, img.height() as usize, |x, y| (x, y)),
    };
    for x in 0..px_map.width {
        let mut column = px_map.get_column(x);
        column.sort_by_key(|y| 255 - f(img.get_pixel(y.0 as u32, y.1 as u32)));
        for y in 0..px_map.height {
            px_map[y][x] = column[y]
        }
    }
    px_map
}

pub fn pixel_sort(img: &DynamicImage, px_map: &ImgGrid) -> RgbaImage {
    RgbaImage::from_fn(img.width(), img.height(), |x, y| {
        let (x1, y1) = px_map[y as usize][x as usize];
        img.get_pixel(x1 as u32, y1 as u32)
    })
}

pub fn unsort(img: &DynamicImage, px_map: &ImgGrid) -> RgbaImage {
    let mut out_image = RgbaImage::new(img.width(), img.height());
    for y in 0..px_map.height {
        for x in 0..px_map.width {
            let p = img.get_pixel(x as u32, y as u32);
            let (x1, y1) = px_map[y][x];
            out_image.put_pixel(x1 as u32, y1 as u32, p)
        }
    }
    out_image
}

pub(crate) fn draw(
    sort_image: &DynamicImage,
    unsort_image: &DynamicImage,
    dir: SortBy,
    key: SortKey,
) -> RgbaImage {
    let unsort_image = unsort_image.resize_exact(
        sort_image.width(),
        sort_image.height(),
        FilterType::CatmullRom,
    );

    let sort_fn = match key {
        SortKey::Lightness => luma,
        SortKey::Hue => hue,
        SortKey::Saturation => sat,
        SortKey::Red => red,
        SortKey::Green => green,
        SortKey::Blue => blue,
    };

    let px_map = match dir {
        SortBy::Row => pixel_map_row(&sort_image, sort_fn, None),
        SortBy::Column => pixel_map_column(&sort_image, sort_fn, None),
        SortBy::RowCol => {
            let pm = pixel_map_row(&sort_image, sort_fn, None);
            pixel_map_column(&sort_image, sort_fn, Some(pm))
        }
        SortBy::ColRow => {
            let pm = pixel_map_column(&sort_image, sort_fn, None);
            pixel_map_row(&sort_image, sort_fn, Some(pm))
        }
    };
    unsort(&unsort_image, &px_map)
}
