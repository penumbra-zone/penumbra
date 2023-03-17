use regex::Regex;
use reqwest::blocking::Client;

/// The Git LFS server to use.
static GIT_LFS_SERVER: &str =
    "https://github.com/penumbra-zone/penumbra.git/info/lfs/objects/batch";

/// Represents a Git LFS pointer.
pub struct GitLFSPointer {
    /// The unique object ID.
    oid: String,
    /// The hash algorithm used to compute the OID. Only `sha256` is supported.
    hash_algo: String,
    /// The size of the object in bytes.
    size: usize,
}

impl GitLFSPointer {
    /// Parses a Git LFS pointer from raw bytes.
    pub fn parse(bytes: &[u8]) -> Self {
        let pointer_utf8 = std::str::from_utf8(bytes).expect("git LFS should be valid UTF-8");

        // `oid sha256:digest`
        let oid_re = Regex::new(r"oid \w*").unwrap();
        let caps = oid_re
            .captures(pointer_utf8)
            .expect("git LFS pointers have oid field");
        let oid_line: Vec<String> = caps
            .get(1)
            .expect("hash algorithm should be in oid field")
            .as_str()
            .split_whitespace()
            .map(str::to_owned)
            .collect();
        let hash_and_oid: Vec<String> = oid_line[1].split(':').map(str::to_owned).collect();
        let hash_algo = hash_and_oid[0].clone();
        let oid = hash_and_oid[1].clone();

        // `size 12345`
        let size_re = Regex::new(r"size [0-9]*").unwrap();
        let caps = size_re
            .captures(pointer_utf8)
            .expect("git LFS pointers have size field");
        let size_line: Vec<String> = caps
            .get(1)
            .expect("size in bytes should be in git LFS pointer")
            .as_str()
            .split_whitespace()
            .map(str::to_owned)
            .collect();
        let size = size_line[1].parse().expect("size should be a number");

        Self {
            oid,
            hash_algo,
            size,
        }
    }

    /// Resolves the pointer using the Git LFS Batch API.
    /// https://github.com/git-lfs/git-lfs/blob/main/docs/api/batch.md
    pub fn resolve(&self) -> Vec<u8> {
        // Download using Git LFS Batch API
        let request_body = format!(
            r#"{{"operation": "download", "transfer": ["basic"], "objects": [{{"oid": "{}", "size": {}}}]}}"#,
            self.oid, self.size
        );
        let client = Client::new();
        let res = client
            .post(GIT_LFS_SERVER)
            .header("Accept", "application/vnd.git-lfs+json")
            .header("Content-type", "application/vnd.git-lfs+json")
            .body(request_body)
            .send()
            .expect("can get response from Git LFS server");

        // JSON response contains "objects" array -> 0 -> "actions" -> "download" -> "href" which has the
        // actual location of the file.
        let json_res = res
            .json::<serde_json::Value>()
            .expect("result is JSON formatted");

        let href = json_res
            .get("objects")
            .expect("objects key exists")
            .get(0)
            .expect("has at least one entry")
            .get("actions")
            .expect("has actions key")
            .get("download")
            .expect("has download key")
            .get("href")
            .expect("has href key")
            .as_str()
            .expect("can get href from Git LFS response");

        // Actually download that file using the provided URL.
        let res = client.get(href).send().expect("can get file");
        let bytes = res.bytes().expect("can get bytes from file");

        // Check hash locally.
        if self.hash_algo != "sha256" {
            unimplemented!("only sha256 is supported");
        } else {
            use sha2::{Digest, Sha256};
            let sha256_digest = Sha256::digest(&bytes);
            let sha256_str = hex::encode(sha256_digest);
            assert_eq!(sha256_str, self.oid);
        }

        // TODO: Write bytes to location of pointer file so we don't need to download again on
        // next run.

        bytes.into()
    }
}
