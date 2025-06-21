# dedcore
**Oops, no more duplicates!**

## About

dedcore is an intelligent CLI tool for finding and removing duplicate and similar files.

This project aims to provide a robust, safe, and feature-rich deduplication experience for power users and professionals.

## Roadmap

### Core Features
- Multi-Algorithm Hashing: SHA-256, Blake3, and xxHash for different use cases
- Parallel Processing: Rayon-based parallel file processing with progress tracking
- Advanced Filtering: Size ranges, file types, date ranges, regex patterns
- Safe Operations: Quarantine system before actual deletion
- Detailed Reports: JSON/HTML reports with file relationships and savings

### Advanced Features
- Content Similarity: Compare text files using edit distance algorithms
- Image Similarity: Perceptual hashing for images using image crate
- Incremental Scanning: Only scan changed files using modification times and checksums
- Recovery System: Maintain deletion history with rollback capabilities
- Space Analysis: Detailed breakdown of potential space savings

### Advanced Challenges
- Sophisticated Grouping: Group similar files by content, not just exact matches
- Performance Optimization: Memory-mapped files, efficient hash computation
- Advanced Verification: Multiple verification passes before deletion
- Metadata Analysis: Consider file attributes, EXIF data for better deduplication
- Custom Algorithms: Implement domain-specific similarity detection

## Installation

```bash
# Installation instructions coming soon
```

## Usage

```bash
# Usage examples coming soon
```

## License

MIT License