//! Hand-written prost message types for the 4 Futu protos we need.
//!
//! Proto IDs: 1001 (InitConnect), 1004 (KeepAlive),
//!            3004 (GetBasicQot), 3209 (GetOptionChain)

/// Common security identifier.
#[derive(Clone, prost::Message)]
pub struct Security {
    /// Market: 1=HK, 11=US, 21=SH, 22=SZ
    #[prost(int32, required, tag = "1")]
    pub market: i32,
    /// Ticker code, e.g. "SPY"
    #[prost(string, required, tag = "2")]
    pub code: ::prost::alloc::string::String,
}

pub const MARKET_US: i32 = 11;

// ─── Proto 1001: InitConnect ──────────────────────────────────────

pub mod init_connect {
    #[derive(Clone, prost::Message)]
    pub struct C2s {
        #[prost(int32, required, tag = "1")]
        pub client_ver: i32,
        #[prost(string, required, tag = "2")]
        pub client_id: ::prost::alloc::string::String,
        #[prost(bool, optional, tag = "3")]
        pub recv_notify: ::core::option::Option<bool>,
    }

    #[derive(Clone, prost::Message)]
    pub struct S2c {
        #[prost(int32, required, tag = "1")]
        pub server_ver: i32,
        #[prost(uint64, required, tag = "2")]
        pub login_user_id: u64,
        #[prost(uint64, required, tag = "3")]
        pub conn_id: u64,
        #[prost(string, required, tag = "4")]
        pub conn_aes_key: ::prost::alloc::string::String,
        #[prost(int32, required, tag = "5")]
        pub keep_alive_interval: i32,
    }

    #[derive(Clone, prost::Message)]
    pub struct Request {
        #[prost(message, required, tag = "1")]
        pub c2s: C2s,
    }

    #[derive(Clone, prost::Message)]
    pub struct Response {
        #[prost(int32, required, tag = "1")]
        pub ret_type: i32,
        #[prost(string, optional, tag = "2")]
        pub ret_msg: ::core::option::Option<::prost::alloc::string::String>,
        #[prost(int32, optional, tag = "3")]
        pub err_code: ::core::option::Option<i32>,
        #[prost(message, optional, tag = "4")]
        pub s2c: ::core::option::Option<S2c>,
    }
}

// ─── Proto 1004: KeepAlive ────────────────────────────────────────

pub mod keep_alive {
    #[derive(Clone, prost::Message)]
    pub struct C2s {
        #[prost(int64, required, tag = "1")]
        pub time: i64,
    }

    #[derive(Clone, prost::Message)]
    pub struct S2c {
        #[prost(int64, required, tag = "1")]
        pub time: i64,
    }

    #[derive(Clone, prost::Message)]
    pub struct Request {
        #[prost(message, required, tag = "1")]
        pub c2s: C2s,
    }

    #[derive(Clone, prost::Message)]
    pub struct Response {
        #[prost(int32, required, tag = "1")]
        pub ret_type: i32,
        #[prost(string, optional, tag = "2")]
        pub ret_msg: ::core::option::Option<::prost::alloc::string::String>,
        #[prost(int32, optional, tag = "3")]
        pub err_code: ::core::option::Option<i32>,
        #[prost(message, optional, tag = "4")]
        pub s2c: ::core::option::Option<S2c>,
    }
}

// ─── Proto 3004: GetBasicQot ──────────────────────────────────────

pub mod get_basic_qot {
    use super::Security;

    #[derive(Clone, prost::Message)]
    pub struct C2s {
        #[prost(message, repeated, tag = "1")]
        pub security_list: ::prost::alloc::vec::Vec<Security>,
    }

    /// Option-specific fields within BasicQot.
    #[derive(Clone, prost::Message)]
    pub struct OptionBasicQotExData {
        #[prost(double, required, tag = "1")]
        pub strike_price: f64,
        #[prost(int32, required, tag = "2")]
        pub contract_size: i32,
        #[prost(int32, required, tag = "3")]
        pub open_interest: i32,
        #[prost(double, required, tag = "4")]
        pub implied_volatility: f64,
        #[prost(double, required, tag = "5")]
        pub premium: f64,
        #[prost(double, required, tag = "6")]
        pub delta: f64,
        #[prost(double, required, tag = "7")]
        pub gamma: f64,
        #[prost(double, required, tag = "8")]
        pub vega: f64,
        #[prost(double, required, tag = "9")]
        pub theta: f64,
        #[prost(double, required, tag = "10")]
        pub rho: f64,
    }

    #[derive(Clone, prost::Message)]
    pub struct BasicQot {
        #[prost(message, required, tag = "1")]
        pub security: Security,
        #[prost(double, required, tag = "9")]
        pub cur_price: f64,
        #[prost(int64, required, tag = "11")]
        pub volume: i64,
        #[prost(message, optional, tag = "16")]
        pub option_ex_data: ::core::option::Option<OptionBasicQotExData>,
    }

    #[derive(Clone, prost::Message)]
    pub struct S2c {
        #[prost(message, repeated, tag = "1")]
        pub basic_qot_list: ::prost::alloc::vec::Vec<BasicQot>,
    }

    #[derive(Clone, prost::Message)]
    pub struct Request {
        #[prost(message, required, tag = "1")]
        pub c2s: C2s,
    }

    #[derive(Clone, prost::Message)]
    pub struct Response {
        #[prost(int32, required, tag = "1")]
        pub ret_type: i32,
        #[prost(string, optional, tag = "2")]
        pub ret_msg: ::core::option::Option<::prost::alloc::string::String>,
        #[prost(int32, optional, tag = "3")]
        pub err_code: ::core::option::Option<i32>,
        #[prost(message, optional, tag = "4")]
        pub s2c: ::core::option::Option<S2c>,
    }
}

// ─── Proto 3209: GetOptionChain ───────────────────────────────────

pub mod get_option_chain {
    use super::Security;

    #[derive(Clone, prost::Message)]
    pub struct C2s {
        #[prost(message, required, tag = "1")]
        pub owner: Security,
        #[prost(string, required, tag = "4")]
        pub begin_time: ::prost::alloc::string::String,
        #[prost(string, required, tag = "5")]
        pub end_time: ::prost::alloc::string::String,
    }

    #[derive(Clone, prost::Message)]
    pub struct OptionStaticExData {
        /// 1=Call, 2=Put
        #[prost(int32, required, tag = "1")]
        pub r#type: i32,
        #[prost(message, required, tag = "2")]
        pub owner: Security,
        #[prost(string, required, tag = "3")]
        pub strike_time: ::prost::alloc::string::String,
        #[prost(double, required, tag = "4")]
        pub strike_price: f64,
        #[prost(bool, required, tag = "5")]
        pub suspend: bool,
    }

    #[derive(Clone, prost::Message)]
    pub struct SecurityStaticBasic {
        #[prost(message, required, tag = "1")]
        pub security: Security,
        #[prost(int64, required, tag = "2")]
        pub id: i64,
        #[prost(int32, required, tag = "3")]
        pub lot_size: i32,
        #[prost(int32, required, tag = "4")]
        pub sec_type: i32,
        #[prost(string, required, tag = "5")]
        pub name: ::prost::alloc::string::String,
        #[prost(string, required, tag = "6")]
        pub list_time: ::prost::alloc::string::String,
    }

    #[derive(Clone, prost::Message)]
    pub struct SecurityStaticInfo {
        #[prost(message, required, tag = "1")]
        pub basic: SecurityStaticBasic,
        #[prost(message, optional, tag = "3")]
        pub option_ex_data: ::core::option::Option<OptionStaticExData>,
    }

    #[derive(Clone, prost::Message)]
    pub struct OptionItem {
        #[prost(message, optional, tag = "1")]
        pub call: ::core::option::Option<SecurityStaticInfo>,
        #[prost(message, optional, tag = "2")]
        pub put: ::core::option::Option<SecurityStaticInfo>,
    }

    #[derive(Clone, prost::Message)]
    pub struct OptionChain {
        #[prost(string, required, tag = "1")]
        pub strike_time: ::prost::alloc::string::String,
        #[prost(message, repeated, tag = "2")]
        pub option: ::prost::alloc::vec::Vec<OptionItem>,
    }

    #[derive(Clone, prost::Message)]
    pub struct S2c {
        #[prost(message, repeated, tag = "1")]
        pub option_chain: ::prost::alloc::vec::Vec<OptionChain>,
    }

    #[derive(Clone, prost::Message)]
    pub struct Request {
        #[prost(message, required, tag = "1")]
        pub c2s: C2s,
    }

    #[derive(Clone, prost::Message)]
    pub struct Response {
        #[prost(int32, required, tag = "1")]
        pub ret_type: i32,
        #[prost(string, optional, tag = "2")]
        pub ret_msg: ::core::option::Option<::prost::alloc::string::String>,
        #[prost(int32, optional, tag = "3")]
        pub err_code: ::core::option::Option<i32>,
        #[prost(message, optional, tag = "4")]
        pub s2c: ::core::option::Option<S2c>,
    }
}
