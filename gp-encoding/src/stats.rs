use std::fmt;

#[derive(Debug, Clone)]
pub struct BandStats {
    pub band_index: u32,
    pub dtype_name: String,
    pub total_cells: u64,
    pub valued_cells: u64,
    pub fill_cells: u64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
    pub sum_sq: f64,
    pub histogram: Vec<(String, u64)>,
}

impl BandStats {
    pub fn mean(&self) -> f64 {
        if self.valued_cells == 0 {
            0.0
        } else {
            self.sum / self.valued_cells as f64
        }
    }

    pub fn stddev(&self) -> f64 {
        if self.valued_cells <= 1 {
            0.0
        } else {
            let n = self.valued_cells as f64;
            let variance = (self.sum_sq - (self.sum * self.sum) / n) / (n - 1.0);
            variance.max(0.0).sqrt()
        }
    }

    pub fn fill_percentage(&self) -> f64 {
        if self.total_cells == 0 {
            0.0
        } else {
            (self.fill_cells as f64 / self.total_cells as f64) * 100.0
        }
    }

    pub fn valued_percentage(&self) -> f64 {
        if self.total_cells == 0 {
            0.0
        } else {
            (self.valued_cells as f64 / self.total_cells as f64) * 100.0
        }
    }
}

/// Aggregated statistics for the full conversion.
#[derive(Debug, Clone)]
pub struct ConversionReport {
    pub num_chunks: u64,
    pub chunk_size: u64,
    pub refinement_level: u32,
    pub bands: Vec<BandStats>,
}

pub struct BandStatsCollector {
    band_index: u32,
    dtype_name: String,
    total_cells: u64,
    valued_cells: u64,
    min: f64,
    max: f64,
    sum: f64,
    sum_sq: f64,
    values: Vec<f64>,
}

impl BandStatsCollector {
    pub fn new(band_index: u32, dtype_name: String) -> Self {
        Self {
            band_index,
            dtype_name,
            total_cells: 0,
            valued_cells: 0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            sum: 0.0,
            sum_sq: 0.0,
            values: Vec::new(),
        }
    }

    pub fn record_value(&mut self, value: f64) {
        self.valued_cells += 1;
        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value;
        }
        self.sum += value;
        self.sum_sq += value * value;
        self.values.push(value);
    }

    pub fn set_total_cells(&mut self, total: u64) {
        self.total_cells = total;
    }

    pub fn finish(self) -> BandStats {
        let histogram = if self.values.is_empty() {
            Vec::new()
        } else {
            build_histogram(&self.values, self.min, self.max)
        };

        BandStats {
            band_index: self.band_index,
            dtype_name: self.dtype_name,
            total_cells: self.total_cells,
            valued_cells: self.valued_cells,
            fill_cells: self.total_cells.saturating_sub(self.valued_cells),
            min: if self.valued_cells == 0 {
                0.0
            } else {
                self.min
            },
            max: if self.valued_cells == 0 {
                0.0
            } else {
                self.max
            },
            sum: self.sum,
            sum_sq: self.sum_sq,
            histogram,
        }
    }
}

const HISTOGRAM_BUCKETS: usize = 10;

fn build_histogram(values: &[f64], min: f64, max: f64) -> Vec<(String, u64)> {
    if values.is_empty() {
        return Vec::new();
    }

    // If all values are the same, return a single bucket.
    if (max - min).abs() < f64::EPSILON {
        return vec![(format_bucket_value(min), values.len() as u64)];
    }

    let num_buckets = HISTOGRAM_BUCKETS;
    let range = max - min;
    let bucket_width = range / num_buckets as f64;

    let mut counts = vec![0u64; num_buckets];

    for &v in values {
        let idx = ((v - min) / bucket_width) as usize;
        // Clamp the last value into the final bucket.
        let idx = idx.min(num_buckets - 1);
        counts[idx] += 1;
    }

    counts
        .iter()
        .enumerate()
        .map(|(i, &count)| {
            let lo = min + (i as f64) * bucket_width;
            let hi = lo + bucket_width;
            let label = if i == num_buckets - 1 {
                format!("[{}, {}]", format_bucket_value(lo), format_bucket_value(hi))
            } else {
                format!("[{}, {})", format_bucket_value(lo), format_bucket_value(hi))
            };
            (label, count)
        })
        .collect()
}

fn format_bucket_value(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{:.0}", v)
    } else {
        format!("{:.4}", v)
    }
}

// ── Display ──────────────────────────────────────────────────────────────

impl fmt::Display for ConversionReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "╔══════════════════════════════════════════════════════════╗")?;
        writeln!(f, "║              Conversion Statistics Report               ║")?;
        writeln!(f, "╚══════════════════════════════════════════════════════════╝")?;
        writeln!(f)?;
        writeln!(f, "  Refinement level : {}", self.refinement_level)?;
        writeln!(f, "  Chunks           : {}", self.num_chunks)?;
        writeln!(f, "  Chunk size       : {} cells", self.chunk_size)?;
        writeln!(f)?;

        for band in &self.bands {
            writeln!(f, "  ── Band {} ({}) ─────────────────────────────────", band.band_index, band.dtype_name)?;
            writeln!(f)?;
            writeln!(f, "    Total cells    : {:>12}", format_count(band.total_cells))?;
            writeln!(
                f,
                "    Valued cells   : {:>12}  ({:.1}%)",
                format_count(band.valued_cells),
                band.valued_percentage()
            )?;
            writeln!(
                f,
                "    Empty cells     : {:>12}  ({:.1}%)",
                format_count(band.fill_cells),
                band.fill_percentage()
            )?;
            writeln!(f)?;

            if band.valued_cells > 0 {
                writeln!(f, "    Min            : {:>12}", format_stat_value(band.min))?;
                writeln!(f, "    Max            : {:>12}", format_stat_value(band.max))?;
                writeln!(f, "    Mean           : {:>12}", format_stat_value(band.mean()))?;
                writeln!(f, "    Std dev        : {:>12}", format_stat_value(band.stddev()))?;
                writeln!(f)?;

                // if !band.histogram.is_empty() {
                //     writeln!(f, "    Value distribution:")?;
                //     let max_count = band.histogram.iter().map(|(_, c)| *c).max().unwrap_or(1);
                //     let bar_max_width = 24;
                //     for (label, count) in &band.histogram {
                //         let bar_len = if max_count > 0 {
                //             (*count as f64 / max_count as f64 * bar_max_width as f64).ceil()
                //                 as usize
                //         } else {
                //             0
                //         };
                //         let bar: String = "█".repeat(bar_len);
                //         let pct = if band.valued_cells > 0 {
                //             *count as f64 / band.valued_cells as f64 * 100.0
                //         } else {
                //             0.0
                //         };
                //         writeln!(
                //             f,
                //             "      {:<22} {:>8} {:>5.1}%  {}",
                //             label,
                //             format_count(*count),
                //             pct,
                //             bar
                //         )?;
                //     }
                //     writeln!(f)?;
                // }
            } else {
                writeln!(f, "    (no valued cells)")?;
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

fn format_count(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

fn format_stat_value(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{:.0}", v)
    } else {
        format!("{:.6}", v)
    }
}
