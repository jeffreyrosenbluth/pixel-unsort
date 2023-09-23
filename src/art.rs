use std::ops::Neg;

use image::imageops::FilterType;
use image::*;

use crate::matrix::*;
use crate::sortfns::*;
use rayon::prelude::*;

type ImgGrid = Matrix<(usize, usize)>;

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone, Copy)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    fn dir(self) -> i16 {
        match self {
            SortOrder::Ascending => 1,
            SortOrder::Descending => -1,
        }
    }
}

impl Neg for SortOrder {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}

pub fn pixel_map_row(
    img: &DynamicImage,
    f: SortFn,
    order: SortOrder,
    grid: Option<ImgGrid>,
) -> ImgGrid {
    let mut px_map = match grid {
        Some(g) => g,
        None => Matrix::generate(img.width() as usize, img.height() as usize, |x, y| (x, y)),
    };
    for y in 0..px_map.height {
        let mut row = px_map[y].to_vec();
        row.sort_by_key(|x| order.dir() * f(img.get_pixel(x.0 as u32, x.1 as u32)));
        let mut indices = (0..row.len()).collect::<Vec<_>>();
        indices.sort_by_key(|i| row[*i].0);
        let row1 = indices.iter().map(|i| (*i, y)).collect::<Vec<_>>();
        for (i, e) in px_map[y].iter_mut().enumerate() {
            *e = row1[i];
        }
    }
    px_map
}

pub fn pixel_map_column(
    img: &DynamicImage,
    f: SortFn,
    order: SortOrder,
    grid: Option<ImgGrid>,
) -> ImgGrid {
    let mut px_map = match grid {
        Some(g) => g,
        None => Matrix::generate(img.width() as usize, img.height() as usize, |x, y| (x, y)),
    };
    for x in 0..px_map.width {
        let mut column = px_map.get_column(x);
        column.sort_by_key(|y| order.dir() * f(img.get_pixel(y.0 as u32, y.1 as u32)));
        let mut indices = (0..column.len()).collect::<Vec<_>>();
        indices.sort_by_key(|i| column[*i].1);
        let column1 = indices.iter().map(|i| (x, *i)).collect::<Vec<_>>();
        for y in 0..px_map.height {
            px_map[y][x] = column1[y]
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

pub fn pixel_sort_row(img: &DynamicImage, f: SortFn, order: SortOrder) -> RgbaImage {
    let mut data: Vec<u8> = Vec::with_capacity(16 * img.width() as usize * img.height() as usize);
    let buffer = img.to_rgba8();
    for buf_row in buffer.rows() {
        let mut row = Vec::with_capacity(buf_row.len());
        for p in buf_row {
            row.push(*p);
        }
        row.sort_by_key(|p| order.dir() * f(*p));
        for p in row {
            for c in p.channels() {
                data.push(*c);
            }
        }
    }
    ImageBuffer::from_vec(img.width(), img.height(), data).unwrap()
}

pub fn pixel_sort_column(img: &DynamicImage, f: SortFn, order: SortOrder) -> RgbaImage {
    let rotate_img = img.rotate90();
    let sorted_img = pixel_sort_row(&rotate_img, f, -order);
    let dyn_img = DynamicImage::ImageRgba8(sorted_img);
    dyn_img.rotate270().into_rgba8()
}

pub fn pixel_unsort(img: &DynamicImage, px_map: &ImgGrid) -> RgbaImage {
    // let mut buffer: Vec<(usize, usize)> = Vec::with_capacity(&px_map.width * &px_map.height);
    // for i in 0..px_map.height {
    //     for j in 0..px_map.width {
    //         buffer.push((j, i));
    //     }
    // }
    // let mut out_image = RgbaImage::new(img.width(), img.height());
    // let par_iter = buffer.par_iter().map(|p| {
    //     let c = img.get_pixel(p.0 as u32, p.1 as u32);
    //     let (x1, y1) = px_map[p.1 as usize][p.0 as usize];
    //     out_image.put_pixel(x1 as u32, y1 as u32, c);
    // });
    // let img_data: Vec<u8> = par_iter.collect();
    // img_data
    let mut out_image = RgbaImage::new(img.width(), img.height());
    for y in 0..px_map.height {
        for x in 0..px_map.width {
            let (x1, y1) = px_map[y][x];
            let p = img.get_pixel(x1 as u32, y1 as u32);
            out_image.put_pixel(x as u32, y as u32, p)
        }
    }
    out_image
}

pub enum DrawType {
    Sort,
    Unsort,
}

pub(crate) fn draw(
    sort_image: &DynamicImage,
    unsort_image: &DynamicImage,
    dir: SortBy,
    key: SortKey,
    draw_type: DrawType,
    row_sort_order: SortOrder,
    col_sort_order: SortOrder,
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
    };

    let px_map = match dir {
        SortBy::Row => pixel_map_row(sort_image, sort_fn, row_sort_order, None),
        SortBy::Column => pixel_map_column(sort_image, sort_fn, col_sort_order, None),
        SortBy::RowCol => {
            let pm = pixel_map_row(sort_image, sort_fn, row_sort_order, None);
            pixel_map_column(sort_image, sort_fn, col_sort_order, Some(pm))
        }
        SortBy::ColRow => {
            let pm = pixel_map_column(sort_image, sort_fn, col_sort_order, None);
            pixel_map_row(sort_image, sort_fn, row_sort_order, Some(pm))
        }
    };
    match draw_type {
        DrawType::Sort => match dir {
            SortBy::Row => pixel_sort_row(sort_image, sort_fn, row_sort_order),
            SortBy::Column => pixel_sort_column(&sort_image, sort_fn, col_sort_order),
            SortBy::RowCol => {
                let row_sort = pixel_sort_row(sort_image, sort_fn, row_sort_order);
                pixel_sort_column(&DynamicImage::ImageRgba8(row_sort), sort_fn, col_sort_order)
            }
            SortBy::ColRow => {
                let col_sort = pixel_sort_column(sort_image, sort_fn, col_sort_order);
                pixel_sort_row(&DynamicImage::ImageRgba8(col_sort), sort_fn, row_sort_order)
            }
        },
        DrawType::Unsort => pixel_unsort(&unsort_image, &px_map),
    }
}
