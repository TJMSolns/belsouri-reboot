/// The embedded eval license token for Track A development.
/// This token is signed with the dev keypair matching EMBEDDED_PUBLIC_KEY in crypto.rs.
/// Generated: 2026-03-04 via `cargo test generate_dev_keypair -- --nocapture --ignored`
///
/// Payload (decoded):
///   schema_version: 2, license_type: "eval", practice_id: null,
///   issued_at: "2026-01-01T00:00:00Z", max_duration_days: 30,
///   modules: [{name: "scheduling", grace_period_days: 90}]
///
/// REPLACE BEFORE PRODUCTION: sign a real eval payload with the License Server private key.
pub const EVAL_TOKEN: &str = "eyJpc3N1ZWRfYXQiOiIyMDI2LTAxLTAxVDAwOjAwOjAwWiIsImxpY2Vuc2VfdHlwZSI6ImV2YWwiLCJtYXhfZHVyYXRpb25fZGF5cyI6MzAsIm1vZHVsZXMiOlt7ImdyYWNlX3BlcmlvZF9kYXlzIjo5MCwibmFtZSI6InNjaGVkdWxpbmcifV0sInByYWN0aWNlX2lkIjpudWxsLCJzY2hlbWFfdmVyc2lvbiI6Mn0IQVaQEnPV7ONpQkjAOnw4M1dBnJ302wa758w_U-jqDOYz4I2HHXlDUbg8P4XhQ7fQe-x9ECxv3gpiL3rw9s0J";
