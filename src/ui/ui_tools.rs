use std::cmp::Ordering;

/// Creates an array of smoothly mapped values in the range [0-1] based on an array of character counts. Entries with
/// zero counts are mapped to None
pub fn smooth_color_map(counts: [usize; 42]) -> [Option<f32>; 42] {
    let nonzero_values = counts
        .iter()
        .filter(|&&count| count != 0)
        .map(|&count| count as f32)
        .collect::<Vec<f32>>();

    let nonzero_value_count = nonzero_values.len();

    if nonzero_value_count == 0 {
        return [None; 42];
    } else if nonzero_value_count == 1 {
        return counts.map(|count| (count != 0).then_some(0.5));
    }

    let total_work = counts.iter().sum::<usize>();

    let work_per_entry = counts
        .iter()
        .map(|&work| (work as f32) / (total_work as f32))
        .collect::<Vec<f32>>();

    let mut sorted_work_per_entry = work_per_entry
        .clone()
        .into_iter()
        .filter(|&work| work != 0.0)
        .collect::<Vec<f32>>();
    sorted_work_per_entry.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let n = sorted_work_per_entry.len();

    let median_work = if n % 2 == 1 {
        sorted_work_per_entry[n / 2]
    } else {
        let lower_median = sorted_work_per_entry[n / 2 - 1];
        let upper_median = sorted_work_per_entry[n / 2];
        (lower_median + upper_median) / 2.0
    };

    let clamped_work = work_per_entry
        .iter()
        .map(|&work| {
            let clip_threshold = 2.5;

            if work == 0.0 {
                0.0
            } else {
                let lower_threshold = median_work / clip_threshold;
                let upper_threshold = median_work * clip_threshold;

                // TODO: mark clamped and fight it out for the bottom/top 10% or something?
                work.clamp(lower_threshold, upper_threshold)
            }
        })
        .collect::<Vec<f32>>();

    let (min_work, max_work) = clamped_work
        .iter()
        .fold((clamped_work[0], clamped_work[0]), |(min, max), &val| {
            (min.min(val), max.max(val))
        });

    let mut smoothed = [None; 42];

    for (i, &work) in clamped_work.iter().enumerate() {
        if work == 0.0 {
            continue;
        }

        let normalized_work = (work - min_work) / (max_work - min_work);

        smoothed[i] = Some(normalized_work);
    }

    smoothed
}
