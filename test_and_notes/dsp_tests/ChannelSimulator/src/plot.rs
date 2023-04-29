use plotters::prelude::*;

// TODO:
// + substitute f32 with some generis over FloatingPoint
// + make everything more efficient
pub fn plot(
    title: &str,
    caption: &str,
    x: Vec<f32>,
    y: Vec<f32>,
    x_min_max: Option<(Option<f32>, Option<f32>)>,
    y_min_max: Option<(Option<f32>, Option<f32>)>,
) -> Result<(), &'static str> {
    // Find min and max if not specified

    // Ordering float -> complex because NaN does not allow a complete ordering, ORD is not possible
    let comparison = |a: &&f32, b: &&f32| a.total_cmp(b);

    let find_min_max = |axis: &Vec<f32>,
                        min_max: Option<(Option<f32>, Option<f32>)>|
     -> Result<(f32, f32), &'static str> {
        match min_max {
            None | Some((None, None)) => Ok((
                *axis.iter().min_by(comparison).ok_or("Impossible min")?,
                *axis
                    .into_iter()
                    .max_by(comparison)
                    .ok_or("Impossible max")?,
            )),
            Some((Some(min), None)) => Ok((
                min,
                *axis.iter().max_by(comparison).ok_or("Impossible max")?,
            )),
            Some((None, Some(max))) => Ok((
                *axis
                    .into_iter()
                    .min_by(comparison)
                    .ok_or("Impossible min")?,
                max,
            )),
            Some((Some(min), Some(max))) => Ok((min, max)),
        }
    };

    // define x_min, x_max, y_min, y_max
    let (x_min, x_max) = find_min_max(&x, x_min_max)?;
    let (y_min, y_max) = find_min_max(&y, y_min_max)?;

    let path = format!("{}.png", title);

    let root =
        BitMapBackend::new(std::path::Path::new(path.as_str()), (640, 480)).into_drawing_area();
    root.fill(&WHITE)
        .map_err(|_| "Impossible fill backgorung")?;

    let mut chart = ChartBuilder::on(&root)
        .caption(caption, ("sans-serif", 25).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|_| "Impossible create chart")?;

    chart
        .configure_mesh()
        .draw()
        .map_err(|_| "Impossible configure mesh")?;

    chart
        .draw_series(LineSeries::new(x.into_iter().zip(y.into_iter()), &RED))
        .map_err(|_| "Impossibel create Serie")?;
    /*Lables
        .label("y = x^2")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .map_err(|_| "Impossible draw")?;
    */

    root.present().map_err(|_| "Impossible draw")?;
    Ok(())
}
