struct RequestLogLine {
    uuid: String,
    timestamp: String, // (should be iso8601) //

    // -- User and context attributes. -- //
    user_id: Option<u64>,

    // -- http request attributes. -- //
    req_path: String,
    req_method: String,

    // -- Errors attributes. -- //
    client_error_type: Option<String>,
    error_type: Option<String>,
    error_data: Option<Value>,
}
