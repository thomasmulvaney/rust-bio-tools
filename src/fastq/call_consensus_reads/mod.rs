//! Tool remove PCR duplicates from UMI-tagged reads.
//!
//! This tool takes two FASTQ files (forward and reverse)
//! and returns two FASTQ files in which all PCR duplicates
//! have been merged into a consensus read.
//! Duplicates are identified by a Unique Molecular Identifier (UMI).
//!
//! ## Requirements:
//!
//! * starcode
//!
//!
//! ## Usage:
//!
//! ```bash
//! $ rbt call-consensus-reads \
//!   <Path to FASTQ file with forward reads> \
//!   <Path to FASTQ file with reverse reads> \
//!   <Path for output forward FASTQ file> \
//!   <Path for output reverse FASTQ file> \
//!   -l <Length of UMI sequence> \
//!   -D <Maximum distance between sequences in a cluster> \  # See step 1 below
//!   -d <Maximum distance between UMIs in a cluster>   # See step 2 below
//! ```
//!
//! ## Assumptions:
//!
//!  - Reads are of equal length
//!  - UMI is the prefix of the reverse reads
//!
//! ## Workflow:
//!
//! The three main steps are:
//!
//! 1. Cluster all Reads by their sequence
//!
//!    1. Remove UMI sequence from reverse read (and save it for later use)
//!    2. Concatenate forward and reverse sequence
//!    3. Cluster by concatenated sequence using starcode
//!
//!        ```
//!        Forward Read: [================]
//!        Reverse Read: [(UMI)-----------]
//!        Sequence for clustering: [================-----------]
//!        ```
//!   After this, each cluster contains reads with similar sequences,
//!   but potentially different UMIs. Hence, each cluster contains candidates
//!   for merging.
//!
//! 2. For each cluster from step one: Cluster all reads within the cluster
//!    by their UMI (again, using starcode). Each of clusters generated in
//!    This step contain reads with similar read sequences as well as
//!    similar UMIs.
//!    Hence, each (new) cluster contains reads with similar sequence and
//!    UMI which can be merged into a consensus read as PCR duplicates.
//!
//! 3. For each cluster from step two: Compute a consensus sequence.
//!
//!    At each position in the read, all bases and quality values are used
//!    to compute the base with Maximum a-posteriori probability (MAP).
//!
//!      1. For one position, compute the likelihood for the four alleles
//!         A, C, G, and T, incorporating the number of bases as well as
//!         their quality values.
//!      2. Choose the allele with the largest likelihood for the consensus read.
//!      3. Compute the quality value of the consensus read from the maximum posterior
//!         probability used to select the allele.
//!
//! 4. Write consensus reads to output file.
//!
//!
//!
// Since this is a binary crate, documentation needs to be compiled with this 'ancient incantation':
// https://github.com/rust-lang/cargo/issues/1865#issuecomment-394179125
mod calc_consensus;
mod pipeline;

use bio::io::fastq;
use flate2::bufread::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::error::Error;
use std::fs;
use std::io::BufReader;
use std::str;

use pipeline::CallConsensusReads;
use pipeline::CallNonOverlappingConsensusRead;

/// Build readers for the given input and output FASTQ files and pass them to
/// `call_consensus_reads`.
///
/// The type of the readers (writers) depends on the file ending.
/// If the input file names end with '.gz' a gzipped reader (writer) is used.
pub fn call_consensus_reads_from_paths(
    fq1: &str,
    fq2: &str,
    fq1_out: &str,
    fq2_out: &str,
    umi_len: usize,
    seq_dist: usize,
    umi_dist: usize,
) -> Result<(), Box<dyn Error>> {
    eprintln!("Reading input files:\n    {}\n    {}", fq1, fq2);
    eprintln!("Writing output to:\n    {}\n    {}", fq1_out, fq2_out);

    match (fq1.ends_with(".gz"), fq2.ends_with(".gz"), fq1_out.ends_with(".gz"), fq2_out.ends_with(".gz")) {
        (false, false, false, false) => CallNonOverlappingConsensusRead::new(
            &mut fastq::Reader::from_file(fq1)?,
            &mut fastq::Reader::from_file(fq2)?,
            &mut fastq::Writer::to_file(fq1_out)?,
            &mut fastq::Writer::to_file(fq2_out)?,
            umi_len,
            seq_dist,
            umi_dist,
        ).call_consensus_reads(),
        (true, true, false, false) => CallNonOverlappingConsensusRead::new(
            &mut fastq::Reader::new(fs::File::open(fq1).map(BufReader::new).map(GzDecoder::new)?),
            &mut fastq::Reader::new(fs::File::open(fq2).map(BufReader::new).map(GzDecoder::new)?),
            &mut fastq::Writer::to_file(fq1_out)?,
            &mut fastq::Writer::to_file(fq2_out)?,
            umi_len,
            seq_dist,
            umi_dist,
        ).call_consensus_reads(),
        (false, false, true, true) => CallNonOverlappingConsensusRead::new(
            &mut fastq::Reader::from_file(fq1)?,
            &mut fastq::Reader::from_file(fq2)?,
            &mut fastq::Writer::new(GzEncoder::new(fs::File::create(fq1_out)?, Compression::default())),
            &mut fastq::Writer::new(GzEncoder::new(fs::File::create(fq2_out)?, Compression::default())),
            umi_len,
            seq_dist,
            umi_dist,
        ).call_consensus_reads(),
        (true, true, true, true) => CallNonOverlappingConsensusRead::new(
            &mut fastq::Reader::new(fs::File::open(fq1).map(BufReader::new).map(GzDecoder::new)?),
            &mut fastq::Reader::new(fs::File::open(fq2).map(BufReader::new).map(GzDecoder::new)?),
            &mut fastq::Writer::new(GzEncoder::new(fs::File::create(fq1_out)?, Compression::default())),
            &mut fastq::Writer::new(GzEncoder::new(fs::File::create(fq2_out)?, Compression::default())),
            umi_len,
            seq_dist,
            umi_dist,
        ).call_consensus_reads(),
        _ => panic!("Invalid combination of files. Each pair of files (input and output) need to be both gzipped or both not zipped.")
    }
}
