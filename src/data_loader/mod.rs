pub mod io;
mod constructor;
mod examples_in_mem;

use rand;
use rand::Rng;

use std::fs::File;
use std::io::BufReader;

use commons::max;
use commons::get_weight;
use commons::get_weights;
use commons::is_positive;
use commons::get_symmetric_label;
use commons::TLabel;
use commons::Example;
use commons::Model;
use commons::PerformanceMonitor;

use self::constructor::Constructor;
use self::examples_in_mem::Examples;
use self::io::*;


#[derive(Debug, PartialEq, Eq)]
pub enum Format {
    Binary,
    Text,
    InMemory
}

#[derive(Debug)]
enum Reader {
    DiskReader(BufReader<File>),
    MemReader(Examples)
}

#[derive(Debug)]
pub struct DataLoader {
    name: String,
    filename: String,
    size: usize,
    feature_size: usize,
    batch_size: usize,
    num_batch: usize,
    format: Format,
    bytes_per_example: usize,
    binary_constructor: Option<Constructor>,

    num_positive: usize,
    num_negative: usize,
    sum_weights: f32,
    sum_weight_squared: f32,
    ess: Option<f32>,

    _reader: Reader,
    _curr_loc: usize,
    _cursor: usize,
    _curr_batch: Vec<Example>,
    _scores_synced: bool,

    base_node: usize,
    scores_version: Vec<usize>,
    base_scores: Vec<f32>,
    scores: Vec<f32>,
    relative_scores: Vec<f32>,

    loss_estimate: f32,
    load_performance: PerformanceMonitor,
    scores_performance: PerformanceMonitor
}

// TODO: write scores to disk
impl DataLoader {
    fn new(name: String, filename: String, examples: Option<Examples>,
           size: usize, feature_size: usize, batch_size: usize,
           format: Format, bytes_per_example: usize, base_node: usize,
           scores: Vec<f32>) -> DataLoader {
        assert!(batch_size <= size);
        let num_batch = size / batch_size + ((size % batch_size > 0) as usize);
        let relative_scores = vec![0.0; size];
        let reader = if format == Format::InMemory {
            Reader::MemReader(examples.unwrap())
        } else {
            Reader::DiskReader(create_bufreader(&filename))
        };
        debug!(
            "new-data-loader, {}, {:?}, {}, {}, {}, {}, {}",
            filename, format, size, feature_size, batch_size, base_node, bytes_per_example
        );
        DataLoader {
            name: name,
            filename: filename,
            size: size,
            feature_size: feature_size,
            batch_size: batch_size,
            num_batch: num_batch,
            format: format,
            bytes_per_example: bytes_per_example,
            binary_constructor: None,

            num_positive: 0,
            num_negative: 0,
            sum_weights: 0.0,
            sum_weight_squared: 0.0,
            ess: None,

            _reader: reader,
            _curr_loc: 0,
            _cursor: 0,
            _curr_batch: vec![],
            _scores_synced: false,

            base_node: 0,
            scores_version: vec![base_node; num_batch],
            base_scores: scores.clone(),
            scores: scores,
            relative_scores: relative_scores,

            loss_estimate: 1.0,
            load_performance: PerformanceMonitor::new(),
            scores_performance: PerformanceMonitor::new(),
        }
    }

    pub fn from_scratch(name: String, filename: String, size: usize, feature_size: usize,
                        batch_size: usize, format: Format, bytes_per_example: usize) -> DataLoader {
        assert!(format != Format::InMemory);
        let mut ret = DataLoader::new(name, filename, None, size, feature_size, batch_size,
                                      format, bytes_per_example, 0, vec![0.0; size]);
        if ret.format == Format::Text {
            ret.binary_constructor = Some(Constructor::new(size, false));
        }
        ret
    }

    pub fn from_constructor(&self, name: String, constructor: Constructor,
                            base_node: usize) -> DataLoader {
        let (filename, some_examples, mut scores, size, bytes_per_example)
                :(String, Option<Examples>, Vec<f32>, usize, usize) = constructor.get_content();
        assert!(some_examples.is_some());
        scores.shrink_to_fit();
        let mut examples = some_examples.unwrap();
        examples.shrink_to_fit();
        DataLoader::new(
            name,
            filename,
            Some(examples),
            size,
            self.feature_size,
            self.batch_size,
            Format::InMemory,
            bytes_per_example,
            base_node,
            scores
        )
    }

    #[allow(dead_code)]
    pub fn get_feature_size(&self) -> usize {
        self.feature_size
    }

    pub fn get_ess(&self) -> Option<f32> {
        self.ess
    }

    pub fn get_curr_batch(&self) -> &Vec<Example> {
        &self._curr_batch
    }

    pub fn get_relative_scores(&self) -> &[f32] {
        assert!(self._scores_synced);
        &self.relative_scores.as_slice()
    }

    pub fn get_absolute_scores(&self) -> &[f32] {
        assert!(self._scores_synced);
        let head = self._curr_loc * self.batch_size;
        let tail = head + self._curr_batch.len();
        &self.scores[head..tail]
    }

    pub fn get_num_examples(&self) -> usize {
        self.size
    }

    pub fn get_num_batches(&self) -> usize {
        self.num_batch
    }

    pub fn fetch_next_batch(&mut self) {
        self.load_performance.resume();

        let mut loader_reset = false;
        self._curr_loc = self._cursor;
        let batch_size = if (self._cursor + 1) * self.batch_size < self.size {
            self._cursor += 1;
            self.batch_size
        } else {
            loader_reset = true;
            let tail_remains = self.size - self._cursor * self.batch_size;
            self._cursor = 0;
            tail_remains
        };
        self._curr_batch =
            match self._reader {
                Reader::DiskReader(ref mut reader) => {
                    if self.format == Format::Text {
                        let batch: Vec<Example> =
                            read_k_labeled_data(reader, batch_size, 0 as TLabel, self.feature_size);
                        if let Some(ref mut constructor) = self.binary_constructor {
                            batch.iter().for_each(|data| {
                                constructor.append_data(data, 0.0);
                            });
                        }
                        batch
                    } else {
                        read_k_labeled_data_from_binary_file(
                            reader, batch_size, self.bytes_per_example)
                    }
                },
                Reader::MemReader(ref mut examples) => {
                    examples.fetch(batch_size)
                }
            };
        self._scores_synced = false;

        if loader_reset {
            // switch to binary
            if let Some(ref constructor) = self.binary_constructor {
                self.format = Format::Binary;
                self.filename = constructor.get_filename();
                self.bytes_per_example = constructor.get_bytes_per_example();
                info!("Text-based loader `{}` has been converted to Binary-based. \
                      Filename: {}, bytes_per_example: {}.",
                      self.name, constructor.get_filename(), constructor.get_bytes_per_example());
            }
            self.binary_constructor = None;
            // update ESS
            let count = self.num_positive + self.num_negative;
            let ess = self.sum_weights.powi(2) / self.sum_weight_squared / (count as f32);
            debug!("loader-reset, {}, {}, {}, {}",
                   self.name, self.num_positive, self.num_negative, ess);
            self.ess = Some(ess);
            self.num_positive = 0;
            self.num_negative = 0;
            self.sum_weights = 0.0;
            self.sum_weight_squared = 0.0;
            // reset bufreader
            self.set_bufrader();
        }

        self.load_performance.update(self._curr_batch.len());
        self.load_performance.pause();
        let (since_last_check, _, _, speed) = self.load_performance.get_performance();
        if since_last_check >= 120 {
            debug!("loader-loading-speed, {}, {:?}, {}", self.name, self.format, speed);
            self.load_performance.reset_last_check();
        }
    }

    pub fn fetch_scores(&mut self, trees: &Model) {
        if self._scores_synced {
            return;
        }
        self.scores_performance.resume();

        let tree_head = self.scores_version[self._curr_loc];
        let tree_tail = trees.len();
        let head = self._curr_loc * self.batch_size;
        let tail = head + self._curr_batch.len();

        {
            let scores_region = &mut self.scores[head..tail];
            for tree in trees[tree_head..tree_tail].iter() {
                tree.add_prediction_to_score(&self._curr_batch, scores_region)
            }
        }
        self.relative_scores = self.scores[head..tail]
                                   .iter()
                                   .zip(self.base_scores[head..tail].iter())
                                   .map(|(a, b)| a - b)
                                   .collect();
        self.scores_version[self._curr_loc] = tree_tail;
        self._scores_synced = true;
        self.update_stats_for_ess();

        self.scores_performance.update(tail - head);
        self.scores_performance.pause();
        let (since_last_check, _, _, speed) = self.scores_performance.get_performance();

        let rou = 1.0 / self.num_batch as f32;
        // adaboost
        // let mut new_avg: f32 = get_weights(&self._curr_batch, &self.relative_scores).iter().sum();
        // logitboost
        let mut new_avg: f32 = get_weights(&self._curr_batch, &self.relative_scores).iter()
                                    .map(|t| (1.0 / t).ln()).sum();
        new_avg = new_avg / (self._curr_batch.len() as f32);
        self.loss_estimate = self.loss_estimate * (1.0 - rou) + new_avg * rou;

        if since_last_check >= 10 {
            debug!("loader-scoring-stats, {}, {:?}, {}, {}", self.name, self.format, speed, self.loss_estimate);
            self.scores_performance.reset_last_check();
        }
    }

    fn update_stats_for_ess(&mut self) {
        let mut num_positive       = 0;
        let mut num_negative       = 0;
        let mut sum_weights        = 0.0;
        let mut sum_weight_squared = 0.0;
        self._curr_batch
            .iter()
            .zip(self.relative_scores.iter())
            .for_each(|(data, score)| {
                if is_positive(&get_symmetric_label(data)) {
                    num_positive += 1;
                } else {
                    num_negative += 1;
                }
                let w = get_weight(data, *score);
                sum_weights += w;
                sum_weight_squared += w * w;
            });
        self.num_positive       += num_positive;
        self.num_negative       += num_negative;
        self.sum_weights        += sum_weights;
        self.sum_weight_squared += sum_weight_squared;
    }

    fn set_bufrader(&mut self) {
        if let Reader::MemReader(ref mut examples) = self._reader {
            examples.reset();
        } else {
            self._reader = Reader::DiskReader(create_bufreader(&self.filename));
        }
    }

    // TODO: implement stratified sampling version
    pub fn sample(&mut self, trees: &Model, sample_ratio: f32,
                  sampler_timer: &mut PerformanceMonitor) -> DataLoader {
        let mut timer = PerformanceMonitor::new();
        timer.start();
        sampler_timer.resume();

        info!("Sampling started. Sample ratio is {}. Data size is {}.", sample_ratio, self.size);
        let (interval, size) = self.get_estimated_interval_and_size(trees, sample_ratio, sampler_timer);
        info!("Sample size is estimated to be {}. Interval is {}.", size, interval);

        let mut sum_weights = (rand::thread_rng().gen::<f32>()) * interval;
        let mut constructor = Constructor::new(size, true);
        let mut max_repeat = 0;
        for _ in 0..self.num_batch {
            self.fetch_next_batch();
            self.fetch_scores(trees);
            let data = self.get_curr_batch();
            self.get_absolute_scores()
                .iter()
                .zip(data.iter())
                .for_each(|(score, data)| {
                    let w = get_weight(data, *score);
                    let next_sum_weight = sum_weights + w;
                    let num_copies =
                        (next_sum_weight / interval) as usize - (sum_weights / interval) as usize;
                    max_repeat = max(max_repeat, num_copies);
                    (0..num_copies).for_each(|_| {
                        constructor.append_data(data, *score);
                    });
                    sum_weights = next_sum_weight - num_copies as f32 * interval;
                });
            sampler_timer.update(data.len());
        }

        sampler_timer.pause();

        let ret = self.from_constructor(self.name.clone() + " sample", constructor, trees.len());
        debug!("sampling-finished, {}, {}, {}",
               timer.get_duration(), ret.get_num_examples(), max_repeat);
        ret
    }

    fn get_estimated_interval_and_size(&mut self, trees: &Model, sample_ratio: f32,
                                       sampler_timer: &mut PerformanceMonitor) -> (f32, usize) {
        let mut sum_weights = 0.0;
        let mut max_weight = 0.0;
        let mut num_scanned = 0;
        while num_scanned <= 500000 {
        // for _ in 0..self.num_batch {
            self.fetch_next_batch();
            self.fetch_scores(trees);
            let data = self.get_curr_batch();
            let scores = self.get_absolute_scores();
            let ws = get_weights(&data, &scores);
            let mut local_sum: f32 = 0.0;
            ws.iter().for_each(|w| {
                local_sum += w;
                max_weight = max(max_weight, *w);
            });
            sum_weights += local_sum;
            num_scanned += ws.len();

            sampler_timer.update(data.len());
        }
        let sample_size = (sample_ratio * num_scanned as f32) as usize + 1;
        let interval = sum_weights / (sample_size as f32);
        let max_repeat = max_weight / interval;
        debug!("sample-estimate, {}, {}, {}, {}, {}",
               num_scanned, sum_weights, interval, max_weight, max_repeat);
        (interval, (sample_ratio * self.size as f32) as usize)
    }

    pub fn reset_scores(&mut self) {
        self.scores_version = vec![0; self.num_batch];
        self.scores = vec![0.0; self.scores.len()];
    }
}
