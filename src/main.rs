//! Documentation for Rust Bio Tools
use clap::{load_yaml, value_t};
use log::{error, LevelFilter};

use clap::App;
use fern;
use itertools::Itertools;
use std::process;

pub mod bam;
pub mod bcf;
pub mod fastq;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml)
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    fern::Dispatch::new()
        .format(|out, message, _| out.finish(format_args!("{}", message)))
        .level(if matches.is_present("verbose") {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .chain(std::io::stderr())
        .apply()
        .unwrap();

    if let Some(matches) = matches.subcommand_matches("fastq-split") {
        if let Err(e) = fastq::split::split(&matches.values_of("chunks").unwrap().collect_vec()) {
            error!("{}", e);
            process::exit(1);
        }
    } else if let Some(matches) = matches.subcommand_matches("fastq-filter") {
        if let Err(e) = fastq::filter::filter(&matches.value_of("ids").unwrap()) {
            error!("{}", e);
            process::exit(1);
        }
    } else if let Some(matches) = matches.subcommand_matches("bam-depth") {
        if let Err(e) = bam::depth::depth(
            &matches.value_of("bam-path").unwrap(),
            value_t!(matches, "max-read-length", u32).unwrap_or(1000),
            value_t!(matches, "include-flags", u16).unwrap_or(0),
            value_t!(matches, "exclude-flags", u16).unwrap_or(4 | 256 | 512 | 1024),
            value_t!(matches, "min-mapq", u8).unwrap_or(0),
        ) {
            error!("{}", e);
            process::exit(1);
        }
    } else if let Some(matches) = matches.subcommand_matches("vcf-to-txt") {
        if let Err(e) = bcf::to_txt::to_txt(
            &matches
                .values_of("info")
                .map(|values| values.collect_vec())
                .unwrap_or(vec![]),
            &matches
                .values_of("format")
                .map(|values| values.collect_vec())
                .unwrap_or(vec![]),
            matches.is_present("genotypes"),
        ) {
            error!("{}", e);
            process::exit(1);
        }
    } else if let Some(matches) = matches.subcommand_matches("vcf-match") {
        if let Err(e) = bcf::match_variants::match_variants(
            matches.value_of("vcf").unwrap(),
            value_t!(matches, "max-dist", u32).unwrap_or(20),
            value_t!(matches, "max-len-diff", u32).unwrap_or(10),
        ) {
            error!("{}", e);
            process::exit(1);
        }
    } else if let Some(_) = matches.subcommand_matches("vcf-baf") {
        if let Err(e) = bcf::baf::calculate_baf() {
            error!("{}", e);
            process::exit(1);
        }
    } else if let Some(matches) = matches.subcommand_matches("call-consensus-reads") {
        if let Err(e) = fastq::call_consensus_reads::call_nonoverlapping_consensus_reads_from_paths(
            matches.value_of("fq1").unwrap(),
            matches.value_of("fq2").unwrap(),
            matches.value_of("consensus-fq1").unwrap(),
            matches.value_of("consensus-fq2").unwrap(),
            value_t!(matches, "umi-len", usize).unwrap(),
            value_t!(matches, "max-seq-dist", usize).unwrap(),
            value_t!(matches, "max-umi-dist", usize).unwrap(),
            matches.is_present("reverse-umi"),
        ) {
            error!("{}", e);
            process::exit(1);
        }
    } else if let Some(matches) = matches.subcommand_matches("call-overlapping-consensus-reads") {
        if let Err(e) = fastq::call_consensus_reads::call_overlapping_consensus_reads_from_paths(
            matches.value_of("fq1").unwrap(),
            matches.value_of("fq2").unwrap(),
            matches.value_of("consensus-fq").unwrap(),
            value_t!(matches, "umi-len", usize).unwrap(),
            value_t!(matches, "max-seq-dist", usize).unwrap(),
            value_t!(matches, "max-umi-dist", usize).unwrap(),
            value_t!(matches, "insert-size", usize).unwrap(),
            value_t!(matches, "std-dev", usize).unwrap(),
            matches.is_present("reverse-umi"),
        ) {
            error!("{}", e);
            process::exit(1);
        }
    }
}
