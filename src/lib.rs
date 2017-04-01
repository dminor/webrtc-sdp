use std::str::FromStr;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::num::ParseIntError;

#[derive(Debug)]
pub enum SdpParserResult {
    ParserLineError   { message: String,
                        line: String },
    ParserUnsupported { message: String,
                        line: String },
    ParserSequence    { message: String,
                        line: Option<usize> },
}

impl From<ParseIntError> for SdpParserResult {
    fn from(_: ParseIntError) -> SdpParserResult {
        // TODO empty line error here makes no sense
        SdpParserResult::ParserLineError { message: "failed to parse integer".to_string(),
                                           line: "".to_string() }
    }
}

#[derive(Clone)]
pub enum SdpAttributeType {
    // TODO consolidate these into groups
    Candidate,
    EndOfCandidates,
    Extmap,
    Fingerprint,
    Fmtp,
    Group,
    IceOptions,
    IcePwd,
    IceUfrag,
    Inactive,
    Mid,
    Msid,
    MsidSemantic,
    Rid,
    Recvonly,
    Rtcp,
    RtcpFb,
    RtcpMux,
    RtcpRsize,
    Rtpmap,
    Sctpmap,
    SctpPort,
    Sendonly,
    Sendrecv,
    Setup,
    Simulcast,
    Ssrc,
    SsrcGroup,
}

impl fmt::Display for SdpAttributeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            SdpAttributeType::Candidate => "Candidate",
            SdpAttributeType::EndOfCandidates => "End-Of-Candidates",
            SdpAttributeType::Extmap => "Extmap",
            SdpAttributeType::Fingerprint => "Fingerprint",
            SdpAttributeType::Fmtp => "Fmtp",
            SdpAttributeType::Group => "Group",
            SdpAttributeType::IceOptions => "Ice-Options",
            SdpAttributeType::IcePwd => "Ice-Pwd",
            SdpAttributeType::IceUfrag => "Ice-Ufrag",
            SdpAttributeType::Inactive => "Inactive",
            SdpAttributeType::Mid => "Mid",
            SdpAttributeType::Msid => "Msid",
            SdpAttributeType::MsidSemantic => "Msid-Semantic",
            SdpAttributeType::Rid => "Rid",
            SdpAttributeType::Recvonly => "Recvonly",
            SdpAttributeType::Rtcp => "Rtcp",
            SdpAttributeType::RtcpFb => "Rtcp-Fb",
            SdpAttributeType::RtcpMux => "Rtcp-Mux",
            SdpAttributeType::RtcpRsize => "Rtcp-Rsize",
            SdpAttributeType::Rtpmap => "Rtpmap",
            SdpAttributeType::Sctpmap => "Sctpmap",
            SdpAttributeType::SctpPort => "Sctp-Port",
            SdpAttributeType::Sendonly => "Sendonly",
            SdpAttributeType::Sendrecv => "Sendrecv",
            SdpAttributeType::Setup => "Setup",
            SdpAttributeType::Simulcast => "Simulcast",
            SdpAttributeType::Ssrc => "Ssrc",
            SdpAttributeType::SsrcGroup => "Ssrc-Group",
        };
        write!(f, "{}", printable)
    }
}

#[derive(Clone)]
struct SdpAttributeRtcpFb {
    payload_type: u32,
    // TODO parse this and use an enum instead?
    feedback_type: String
}

#[derive(Clone)]
struct SdpAttributeSctpmap {
    port: u32,
    channels: u32
}

#[derive(Clone)]
struct SdpAttributeRtpmap {
    payload_type: u32,
    codec_name: String,
    frequency: Option<u32>,
    channels: Option<u32>
}

impl SdpAttributeRtpmap {
    pub fn new(pt: u32, codec: String) -> SdpAttributeRtpmap {
        SdpAttributeRtpmap { payload_type: pt,
                             codec_name: codec,
                             frequency: None,
                             channels: None
        }
    }

    fn set_frequency(&mut self, f: u32) {
        self.frequency = Some(f)
    }

    fn set_channels(&mut self, c: u32) {
        self.channels = Some(c)
    }
}

#[derive(Clone)]
enum SdpAttributeSetup {
    active,
    actpass,
    holdconn,
    passive
}

#[derive(Clone)]
enum SdpAttributeValue {
    string {value: String},
    integer {value: u32},
    vector {value: Vec<String>},
    rtpmap {value: SdpAttributeRtpmap},
    rtcpfb {value: SdpAttributeRtcpFb},
    setup {value: SdpAttributeSetup},
    sctpmap {value: SdpAttributeSctpmap},
}

#[derive(Clone)]
pub struct SdpAttribute {
    name: SdpAttributeType,
    string_value: Option<String>,
    value: Option<SdpAttributeValue>
}

impl SdpAttribute {
    pub fn new(t: SdpAttributeType) -> SdpAttribute {
        SdpAttribute { name: t,
                       string_value: None,
                       value: None
                     }
    }

    fn parse_value(&mut self, v: &str) -> Result<(), SdpParserResult> {
        match self.name {
            SdpAttributeType::EndOfCandidates |
            SdpAttributeType::Inactive |
            SdpAttributeType::Recvonly |
            SdpAttributeType::RtcpMux |
            SdpAttributeType::RtcpRsize |
            SdpAttributeType::Sendonly |
            SdpAttributeType::Sendrecv => {
                if v.len() >0 {
                    return Err(SdpParserResult::ParserLineError{
                        message: "This attribute is not allowed to have a value".to_string(),
                        line: v.to_string()})
                }
            },

            SdpAttributeType::IcePwd |
            SdpAttributeType::IceUfrag |
            SdpAttributeType::Mid |
            SdpAttributeType::Rid => {
                self.value = Some(SdpAttributeValue::string {value: v.to_string()})
            },

            SdpAttributeType::IceOptions => {
                self.value = Some(SdpAttributeValue::vector {
                    value: v.split_whitespace().map(|x| x.to_string()).collect()})
            },
            SdpAttributeType::Setup => {
                self.value = Some(SdpAttributeValue::setup {value:
                    match v.to_lowercase().as_ref() {
                        "active" => SdpAttributeSetup::active,
                        "actpass" => SdpAttributeSetup::actpass,
                        "holdconn" => SdpAttributeSetup::holdconn,
                        "passive" => SdpAttributeSetup::passive,
                        _ => return Err(SdpParserResult::ParserLineError{
                            message: "Unsupported setup value".to_string(),
                            line: v.to_string()}),
                    }
                })
            },
            SdpAttributeType::Candidate => (self.string_value = Some(v.to_string())),
            SdpAttributeType::Extmap => (self.string_value = Some(v.to_string())),
            SdpAttributeType::Fingerprint => (self.string_value = Some(v.to_string())),
            SdpAttributeType::Fmtp => (self.string_value = Some(v.to_string())),
            SdpAttributeType::Group => (self.string_value = Some(v.to_string())),
            SdpAttributeType::Msid => (self.string_value = Some(v.to_string())),
            SdpAttributeType::MsidSemantic => (self.string_value = Some(v.to_string())),
            SdpAttributeType::Rtcp => (self.string_value = Some(v.to_string())),
            SdpAttributeType::RtcpFb => {
                let tokens: Vec<&str> = v.splitn(2, ' ').collect();
                self.value = Some(SdpAttributeValue::rtcpfb {value:
                    SdpAttributeRtcpFb {
                        // TODO limit this to dymaic PTs
                        payload_type: try!(tokens[0].parse::<u32>()),
                        feedback_type: tokens[1].to_string()
                    }
                });
            },
            SdpAttributeType::Rtpmap => {
                let tokens: Vec<&str> = v.split_whitespace().collect();
                if tokens.len() != 2 {
                    return Err(SdpParserResult::ParserLineError{
                        message: "Rtpmap needs to have two tokens".to_string(),
                        line: v.to_string()})
                }
                // TODO limit this to dymaic PTs
                let payload_type: u32 = try!(tokens[0].parse::<u32>());
                let split: Vec<&str> = tokens[1].split('/').collect();
                if split.len() > 3 {
                    return Err(SdpParserResult::ParserLineError{
                        message: "Rtpmap codec token can max 3 subtokens".to_string(),
                        line: v.to_string()})
                }
                let mut rtpmap = SdpAttributeRtpmap::new(payload_type,
                                                         split[0].to_string());
                if split.len() > 1 {
                    rtpmap.set_frequency(try!(split[1].parse::<u32>()));
                }
                if split.len() > 2 {
                    rtpmap.set_channels(try!(split[2].parse::<u32>()));
                }
                self.value = Some(SdpAttributeValue::rtpmap {value: rtpmap})
            },
            SdpAttributeType::Sctpmap => {
                let tokens: Vec<&str> = v.split_whitespace().collect();
                if tokens.len() != 3 {
                    return Err(SdpParserResult::ParserLineError{
                        message: "Sctpmap needs to have three tokens".to_string(),
                        line: v.to_string()})
                }
                let port = try!(tokens[0].parse::<u32>());
                if port > 65535 {
                    return Err(SdpParserResult::ParserLineError{
                        message: "Sctpmap port can only be a bit 16bit number".to_string(),
                        line: v.to_string()})
                }
                if tokens[1].to_lowercase() != "webrtc-datachannel" {
                    return Err(SdpParserResult::ParserLineError{
                        message: "Unsupported sctpmap type token".to_string(),
                        line: v.to_string()})
                }
                self.value = Some(SdpAttributeValue::sctpmap {value:
                    SdpAttributeSctpmap {
                        port: port,
                        channels: try!(tokens[2].parse::<u32>())
                    }
                });
            },
            SdpAttributeType::SctpPort => {
                let port = try!(v.parse::<u32>());
                if port > 65535 {
                    return Err(SdpParserResult::ParserLineError{
                        message: "Sctpport port can only be a bit 16bit number".to_string(),
                        line: v.to_string()})
                }
                self.value = Some(SdpAttributeValue::integer {
                    value: port
                })
            }
            SdpAttributeType::Simulcast => (self.string_value = Some(v.to_string())),
            SdpAttributeType::Ssrc => (self.string_value = Some(v.to_string())),
            SdpAttributeType::SsrcGroup => (self.string_value = Some(v.to_string())),
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct SdpBandwidth {
    bwtype: String,
    bandwidth: u64
}

#[derive(Clone)]
enum SdpNetType {
    Internet
}

impl fmt::Display for SdpNetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IN")
    }
}

#[derive(Clone)]
enum SdpAddrType {
    IP4,
    IP6
}

impl fmt::Display for SdpAddrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            SdpAddrType::IP4 => "Ip4",
            SdpAddrType::IP6 => "Ip6"
        };
        write!(f, "{}", printable)
    }
}

#[derive(Clone)]
pub struct SdpConnection {
    nettype: SdpNetType,
    addrtype: SdpAddrType,
    unicast_addr: IpAddr
}

#[derive(Clone,Debug,PartialEq)]
pub enum SdpMediaValue {
    Audio,
    Video,
    Application
}

impl fmt::Display for SdpMediaValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            SdpMediaValue::Audio       => "Audio",
            SdpMediaValue::Video       => "Video",
            SdpMediaValue::Application => "Application"
        };
        write!(f, "{}", printable)
    }
}

#[derive(Clone,Debug,PartialEq)]
pub enum SdpProtocolValue {
    UdpTlsRtpSavpf,
    TcpTlsRtpSavpf,
    DtlsSctp,
    UdpDtlsSctp,
    TcpDtlsSctp
}

impl fmt::Display for SdpProtocolValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            SdpProtocolValue::UdpTlsRtpSavpf => "Udp/Tls/Rtp/Savpf",
            SdpProtocolValue::TcpTlsRtpSavpf => "Tcp/Tls/Rtp/Savpf",
            SdpProtocolValue::DtlsSctp       => "Dtls/Sctp",
            SdpProtocolValue::UdpDtlsSctp    => "Udp/Dtls/Sctp",
            SdpProtocolValue::TcpDtlsSctp    => "Tcp/Dtls/Sctp"
        };
        write!(f, "{}", printable)
    }
}

#[derive(Clone)]
pub enum SdpFormatList {
    Integers {list: Vec<u32>},
    Strings {list: Vec<String>}
}

impl fmt::Display for SdpFormatList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SdpFormatList::Integers { list: ref x } => write!(f, "{:?}", x),
            SdpFormatList::Strings { list: ref x } => write!(f, "{:?}", x)
        }
    }
}

#[derive(Clone)]
pub struct SdpMediaLine {
    pub media: SdpMediaValue,
    pub port: u32,
    pub proto: SdpProtocolValue,
    pub formats: SdpFormatList
}

#[derive(Clone)]
pub struct SdpOrigin {
    username: String,
    session_id: u64,
    session_version: u64,
    nettype: SdpNetType,
    addrtype: SdpAddrType,
    unicast_addr: IpAddr
}

#[derive(Clone)]
pub struct SdpTiming {
    start: u64,
    stop: u64
}

enum SdpLine {
    Attribute {value: SdpAttribute},
    Bandwidth {value: SdpBandwidth},
    Connection {value: SdpConnection},
    Email {value: String},
    Information {value: String},
    Key {value: String},
    Media {value: SdpMediaLine},
    Phone {value: String},
    Origin {value: SdpOrigin},
    Repeat {value: String},
    Session {value: String},
    Timing {value: SdpTiming},
    Uri {value: String},
    Version {value: u64},
    Zone {value: String}
}

pub struct SdpMedia {
    media: SdpMediaLine,
    information: Option<String>,
    connection: Option<SdpConnection>,
    bandwidth: Vec<SdpBandwidth>,
    key: Option<String>,
    attribute: Vec<SdpAttribute>,
}

impl SdpMedia {
    pub fn new(media: SdpMediaLine) -> SdpMedia {
        SdpMedia { media: media,
                   information: None,
                   connection: None,
                   bandwidth: Vec::new(),
                   key: None,
                   attribute: Vec::new()
                 }
    }

    pub fn get_type(&self) -> &SdpMediaValue {
        &self.media.media
    }

    pub fn get_port(&self) -> u32 {
        self.media.port
    }

    pub fn get_proto(&self) -> &SdpProtocolValue {
        &self.media.proto
    }

    pub fn get_formats(&self) -> &SdpFormatList {
        &self.media.formats
    }

    pub fn has_connection(&self) -> bool {
        self.connection.is_some()
    }

    pub fn has_bandwidth(&self) -> bool {
        self.bandwidth.len() > 0
    }

    pub fn has_attributes(&self) -> bool {
        self.attribute.len() > 0
    }

    pub fn add_attribute(&mut self, attr: SdpAttribute) {
        self.attribute.push(attr)
    }

    pub fn add_bandwidth(&mut self, bw: SdpBandwidth) {
        self.bandwidth.push(bw)
    }

    //TODO complain if connection is set already
    pub fn set_connection(&mut self, c: SdpConnection) {
        self.connection = Some(c);
    }

    //TODO complain if information is set already
    pub fn set_information(&mut self, i: String) {
        self.information = Some(i);
    }

    //TODO complain if key is set already
    pub fn set_key(&mut self, k: String) {
        self.key = Some(k);
    }
}

pub struct SdpSession {
    pub version: u64,
    pub origin: SdpOrigin,
    pub session: String,
    information: Option<String>,
    uri: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    pub connection: Option<SdpConnection>,
    pub bandwidth: Vec<SdpBandwidth>,
    pub timing: Option<SdpTiming>,
    repeat: Option<String>,
    zone: Option<String>,
    key: Option<String>,
    pub attribute: Vec<SdpAttribute>,
    pub media: Vec<SdpMedia>,
}

impl SdpSession {
    pub fn new(version: u64, origin: SdpOrigin, session: String) -> SdpSession {
        SdpSession { version: version,
                     origin: origin,
                     session: session,
                     information: None,
                     uri: None,
                     email: None,
                     phone: None,
                     connection: None,
                     bandwidth: Vec::new(),
                     timing: None,
                     repeat: None,
                     zone: None,
                     key: None,
                     attribute: Vec::new(),
                     media: Vec::new()
                   }
    }

    pub fn get_version(&self) -> u64 {
        self.version
    }

    pub fn get_origin(&self) -> &SdpOrigin {
        &self.origin
    }

    pub fn get_session(&self) -> &String {
        &self.session
    }

    pub fn set_information(&mut self, i: String) {
        self.information = Some(i)
    }

    pub fn set_uri(&mut self, u: String) {
        self.uri = Some(u)
    }

    pub fn set_email(&mut self, e: String) {
        self.email = Some(e)
    }

    pub fn set_phone(&mut self, p: String) {
        self.phone = Some(p)
    }

    pub fn set_connection(&mut self, c: SdpConnection) {
        self.connection = Some(c)
    }

    pub fn add_bandwidth(&mut self, b: SdpBandwidth) {
        self.bandwidth.push(b)
    }

    pub fn set_timing(&mut self, t: SdpTiming) {
        self.timing = Some(t)
    }

    pub fn set_repeat(&mut self, r: String) {
        self.repeat = Some(r)
    }

    pub fn set_zone(&mut self, z: String) {
        self.zone = Some(z)
    }

    pub fn set_key(&mut self, k: String) {
        self.key = Some(k)
    }

    pub fn add_attribute(&mut self, a: SdpAttribute) {
        self.attribute.push(a)
    }

    pub fn add_media(&mut self, m: SdpMedia) {
        self.media.push(m)
    }

    pub fn extend_media(&mut self, v: Vec<SdpMedia>) {
        self.media.extend(v)
    }

    pub fn has_timing(&self) -> bool {
        self.timing.is_some()
    }

    pub fn has_attributes(&self) -> bool {
        self.attribute.len() > 0
    }

    pub fn has_media(&self) -> bool {
        self.media.len() > 0
    }
}

fn parse_repeat(value: &str) -> Result<SdpLine, SdpParserResult> {
    // TODO implement this if it's ever needed
    println!("repeat: {}", value);
    Ok(SdpLine::Repeat{value: String::from(value)})
}

#[test]
fn test_repeat_works() {
    // FIXME use a proper r value here
    assert!(parse_repeat("0 0").is_ok());
}

fn parse_zone(value: &str) -> Result<SdpLine, SdpParserResult> {
    // TODO implement this if it's ever needed
    println!("zone: {}", value);
    Ok(SdpLine::Zone {value: String::from(value)})
}

#[test]
fn test_zone_works() {
    // FIXME use a proper z value here
    assert!(parse_zone("0 0").is_ok());
}

fn parse_key(value: &str) -> Result<SdpLine, SdpParserResult> {
    // TODO implement this if it's ever needed
    println!("key: {}", value);
    Ok(SdpLine::Key {value: String::from(value)})
}

#[test]
fn test_keys_works() {
    // FIXME use a proper k value here
    assert!(parse_key("12345").is_ok());
}

fn parse_information(value: &str) -> Result<SdpLine, SdpParserResult> {
    println!("information: {}", value);
    Ok(SdpLine::Information {value: String::from(value)})
}

#[test]
fn test_information_works() {
    assert!(parse_information("foobar").is_ok());
}

fn parse_uri(value: &str) -> Result<SdpLine, SdpParserResult> {
    // TODO check if this is really a URI
    println!("uri: {}", value);
    Ok(SdpLine::Uri {value: String::from(value)})
}

#[test]
fn test_uri_works() {
    assert!(parse_uri("http://www.mozilla.org").is_ok());
}

fn parse_email(value: &str) -> Result<SdpLine, SdpParserResult> {
    // TODO check if this is really an email address
    println!("email: {}", value);
    Ok(SdpLine::Email {value: String::from(value)})
}

#[test]
fn test_email_works() {
    assert!(parse_email("nils@mozilla.com").is_ok());
}

fn parse_phone(value: &str) -> Result<SdpLine, SdpParserResult> {
    // TODO check if this is really a phone number
    println!("phone: {}", value);
    Ok(SdpLine::Phone {value: String::from(value)})
}

#[test]
fn test_phone_works() {
    assert!(parse_phone("+123456789").is_ok());
}

fn parse_session(value: &str) -> Result<SdpLine, SdpParserResult> {
    println!("session: {}", value);
    Ok(SdpLine::Session {value: String::from(value)})
}

#[test]
fn test_session_works() {
    assert!(parse_session("topic").is_ok());
}


fn parse_version(value: &str) -> Result<SdpLine, SdpParserResult> {
    let ver = try!(value.parse::<u64>());
    if ver != 0 {
        return Err(SdpParserResult::ParserLineError {
            message: "unsupported version in v field".to_string(),
            line: value.to_string() });
    };
    println!("version: {}", ver);
    Ok(SdpLine::Version { value: ver })
}

#[test]
fn test_version_works() {
    assert!(parse_version("0").is_ok());
}

#[test]
fn test_version_unsupported_input() {
    assert!(parse_version("1").is_err());
    assert!(parse_version("11").is_err());
    assert!(parse_version("a").is_err());
}

fn parse_nettype(value: &str) -> Result<SdpNetType, SdpParserResult> {
    if value.to_uppercase() != String::from("IN") {
        return Err(SdpParserResult::ParserLineError {
            message: "nettype needs to be IN".to_string(),
            line: value.to_string() });
    };
    Ok(SdpNetType::Internet)
}

fn parse_addrtype(value: &str) -> Result<SdpAddrType, SdpParserResult> {
    Ok(match value.to_uppercase().as_ref() {
        "IP4" => SdpAddrType::IP4,
        "IP6" => SdpAddrType::IP6,
        _ => return Err(SdpParserResult::ParserLineError {
            message: "address type needs to be IP4 or IP6".to_string(),
            line: value.to_string() })
    })
}

fn parse_unicast_addr(addrtype: &SdpAddrType, value: &str) -> Result<IpAddr, SdpParserResult> {
    Ok(match addrtype {
        &SdpAddrType::IP4 => {
            IpAddr::V4(match Ipv4Addr::from_str(value) {
                Ok(n) => n,
                Err(_) => return Err(SdpParserResult::ParserLineError {
                    message: "failed to parse unicast IP4 address attribute".to_string(),
                    line: value.to_string() })
            })
        },
        &SdpAddrType::IP6 => {
            IpAddr::V6(match Ipv6Addr::from_str(value) {
                Ok(n) => n,
                Err(_) => return Err(SdpParserResult::ParserLineError {
                    message: "failed to parse unicast IP6 address attribute".to_string(),
                    line: value.to_string() })
            })
        }
    })
}

fn parse_origin(value: &str) -> Result<SdpLine, SdpParserResult> {
    let ot: Vec<&str> = value.split_whitespace().collect();
    if ot.len() != 6 {
        return Err(SdpParserResult::ParserLineError {
            message: "origin field must have six tokens".to_string(),
            line: value.to_string() });
    }
    let username = ot[0];
    let session_id = try!(ot[1].parse::<u64>());
    let session_version = try!(ot[2].parse::<u64>());
    let nettype = try!(parse_nettype(ot[3]));
    let addrtype = try!(parse_addrtype(ot[4]));
    let unicast_addr = try!(parse_unicast_addr(&addrtype, ot[5]));
    let o = SdpOrigin { username: String::from(username),
                        session_id: session_id,
                        session_version: session_version,
                        nettype: nettype,
                        addrtype: addrtype,
                        unicast_addr: unicast_addr };
    println!("origin: {}, {}, {}, {}, {}, {}",
             o.username, o.session_id, o.session_version, o.nettype,
             o.addrtype, o.unicast_addr);
    Ok(SdpLine::Origin { value: o })
}

#[test]
fn test_origin_works() {
    assert!(parse_origin("mozilla 506705521068071134 0 IN IP4 0.0.0.0").is_ok());
    assert!(parse_origin("mozilla 506705521068071134 0 IN IP6 ::1").is_ok());
}

#[test]
fn test_origin_wrong_amount_of_tokens() {
    assert!(parse_origin("a b c d e").is_err());
    assert!(parse_origin("a b c d e f g").is_err());
}

#[test]
fn test_origin_unsupported_nettype() {
    assert!(parse_origin("mozilla 506705521068071134 0 UNSUPPORTED IP4 0.0.0.0").is_err());
}

#[test]
fn test_origin_unsupported_addrtpe() {
    assert!(parse_origin("mozilla 506705521068071134 0 IN IP1 0.0.0.0").is_err());
}

#[test]
fn test_origin_broken_ip_addr() {
    assert!(parse_origin("mozilla 506705521068071134 0 IN IP4 1.1.1.256").is_err());
    assert!(parse_origin("mozilla 506705521068071134 0 IN IP6 ::g").is_err());
}

#[test]
fn test_origin_addr_type_mismatch() {
    assert!(parse_origin("mozilla 506705521068071134 0 IN IP4 ::1").is_err());
}

fn parse_connection(value: &str) -> Result<SdpLine, SdpParserResult> {
    let cv: Vec<&str> = value.split_whitespace().collect();
    if cv.len() != 3 {
        return Err(SdpParserResult::ParserLineError {
            message: "connection attribute must have three tokens".to_string(),
            line: value.to_string() });
    }
    // TODO this is exactly the same parser as the end of origin.
    //      Share it in a function?!
    let nettype = try!(parse_nettype(cv[0]));
    let addrtype = try!(parse_addrtype(cv[1]));
    let unicast_addr = try!(parse_unicast_addr(&addrtype, cv[2]));
    let c = SdpConnection { nettype: nettype,
                            addrtype: addrtype,
                            unicast_addr: unicast_addr };
    println!("connection: {}, {}, {}",
             c.nettype, c.addrtype, c.unicast_addr);
    Ok(SdpLine::Connection { value: c })
}

#[test]
fn connection_works() {
    assert!(parse_connection("IN IP4 127.0.0.1").is_ok());
}

#[test]
fn connection_lots_of_whitespace() {
    assert!(parse_connection("IN   IP4   127.0.0.1").is_ok());
}

#[test]
fn connection_wrong_amount_of_tokens() {
    assert!(parse_connection("IN IP4").is_err());
    assert!(parse_connection("IN IP4 0.0.0.0 foobar").is_err());
}

#[test]
fn connection_unsupported_nettype() {
    assert!(parse_connection("UNSUPPORTED IP4 0.0.0.0").is_err());
}

#[test]
fn connection_unsupported_addrtpe() {
    assert!(parse_connection("IN IP1 0.0.0.0").is_err());
}

#[test]
fn connection_broken_ip_addr() {
    assert!(parse_connection("IN IP4 1.1.1.256").is_err());
    assert!(parse_connection("IN IP6 ::g").is_err());
}

#[test]
fn connection_addr_type_mismatch() {
    assert!(parse_connection("IN IP4 ::1").is_err());
}

fn parse_bandwidth(value: &str) -> Result<SdpLine, SdpParserResult> {
    let bv: Vec<&str> = value.split(':').collect();
    if bv.len() != 2 {
        return Err(SdpParserResult::ParserLineError {
            message: "bandwidth attribute must have two tokens".to_string(),
            line: value.to_string() });
    }
    let bwtype = bv[0];
    match bwtype.to_uppercase().as_ref() {
        "AS" | "TIAS" => (),
        _ => return Err(SdpParserResult::ParserUnsupported {
              message: "unsupported bandwidth type value".to_string(),
              line: value.to_string() }),
    };
    let bandwidth = try!(bv[1].parse::<u64>());
    let b = SdpBandwidth { bwtype: String::from(bwtype),
                            bandwidth: bandwidth };
    println!("bandwidth: {}, {}",
             b.bwtype, b.bandwidth);
    Ok(SdpLine::Bandwidth { value: b })
}

#[test]
fn bandwidth_works() {
    assert!(parse_bandwidth("TIAS:12345").is_ok());
}

#[test]
fn bandwidth_wrong_amount_of_tokens() {
    assert!(parse_bandwidth("TIAS").is_err());
    assert!(parse_bandwidth("TIAS:12345:xyz").is_err());
}

#[test]
fn bandwidth_unsupported_type() {
    assert!(parse_bandwidth("UNSUPPORTED:12345").is_err());
}

fn parse_timing(value: &str) -> Result<SdpLine, SdpParserResult> {
    let tv: Vec<&str> = value.split_whitespace().collect();
    if tv.len() != 2 {
        return Err(SdpParserResult::ParserLineError {
            message: "timing attribute must have two tokens".to_string(),
            line: value.to_string() });
    }
    let start_time = try!(tv[0].parse::<u64>());
    let stop_time = try!(tv[1].parse::<u64>());
    let t = SdpTiming { start: start_time,
                        stop: stop_time };
    println!("timing: {}, {}", t.start, t.stop);
    Ok(SdpLine::Timing { value: t })
}

#[test]
fn test_timing_works() {
    assert!(parse_timing("0 0").is_ok());
}

#[test]
fn test_timing_non_numeric_tokens() {
    assert!(parse_timing("a 0").is_err());
    assert!(parse_timing("0 a").is_err());
}

#[test]
fn test_timing_wrong_amount_of_tokens() {
    assert!(parse_timing("0").is_err());
    assert!(parse_timing("0 0 0").is_err());
}

fn parse_media_token(value: &str) -> Result<SdpMediaValue, SdpParserResult> {
    Ok(match value.to_lowercase().as_ref() {
        "audio"       => SdpMediaValue::Audio,
        "video"       => SdpMediaValue::Video,
        "application" => SdpMediaValue::Application,
        _ => return Err(SdpParserResult::ParserUnsupported {
              message: "unsupported media value".to_string(),
              line: value.to_string() }),
    })
}

fn parse_protocol_token(value: &str) -> Result<SdpProtocolValue, SdpParserResult> {
    Ok(match value.to_uppercase().as_ref() {
        "UDP/TLS/RTP/SAVPF" => SdpProtocolValue::UdpTlsRtpSavpf,
        "TCP/TLS/RTP/SAVPF" => SdpProtocolValue::TcpTlsRtpSavpf,
        "DTLS/SCTP"         => SdpProtocolValue::DtlsSctp,
        "UDP/DTLS/SCTP"     => SdpProtocolValue::UdpDtlsSctp,
        "TCP/DTLS/SCTP"     => SdpProtocolValue::TcpDtlsSctp,
        _ => return Err(SdpParserResult::ParserUnsupported {
              message: "unsupported protocol value".to_string(),
              line: value.to_string() }),
    })
}

fn parse_media(value: &str) -> Result<SdpLine, SdpParserResult> {
    let mv: Vec<&str> = value.split_whitespace().collect();
    if mv.len() < 4 {
        return Err(SdpParserResult::ParserLineError {
            message: "media attribute must have at least four tokens".to_string(),
            line: value.to_string() });
    }
    let media = try!(parse_media_token(mv[0]));
    let port = try!(mv[1].parse::<u32>());
    if port > 65535 {
        return Err(SdpParserResult::ParserLineError {
            message: "media port token is too big".to_string(),
            line: value.to_string() })
    }
    let proto = try!(parse_protocol_token(mv[2]));
    let fmt_slice: &[&str] = &mv[3..];
    let fmt = match media {
        SdpMediaValue::Audio | SdpMediaValue::Video => {
            let mut fmt_vec: Vec<u32> = vec![];
            for num in fmt_slice {
                let fmt_num = try!(num.parse::<u32>());
                match fmt_num {
                    0 => (),           // PCMU
                    8 => (),           // PCMA
                    9 => (),           // G722
                    13 => (),          // Comfort Noise
                    96 ... 127 => (),  // dynamic range
                    _ => return Err(SdpParserResult::ParserLineError {
                          message: "format number in media line is out of range".to_string(),
                          line: value.to_string() }),
                };
                fmt_vec.push(fmt_num);
            };
            SdpFormatList::Integers { list: fmt_vec }
        },
        SdpMediaValue::Application => {
            let mut fmt_vec: Vec<String> = vec![];
            // TODO enforce length == 1 and content 'webrtc-datachannel' only?
            for token in fmt_slice {
                fmt_vec.push(String::from(*token));
            }
            SdpFormatList::Strings { list: fmt_vec }
        }
    };
    let m = SdpMediaLine { media: media,
                           port: port,
                           proto: proto,
                           formats: fmt };
    println!("media: {}, {}, {}, {}",
             m.media, m.port, m.proto, m.formats);
    Ok(SdpLine::Media { value: m })
}

#[test]
fn test_media_works() {
    assert!(parse_media("audio 9 UDP/TLS/RTP/SAVPF 109").is_ok());
    assert!(parse_media("video 9 UDP/TLS/RTP/SAVPF 126").is_ok());
    assert!(parse_media("application 9 DTLS/SCTP 5000").is_ok());
    assert!(parse_media("application 9 UDP/DTLS/SCTP webrtc-datachannel").is_ok());

    assert!(parse_media("audio 9 UDP/TLS/RTP/SAVPF 109 9 0 8").is_ok());
    assert!(parse_media("audio 0 UDP/TLS/RTP/SAVPF 8").is_ok());
}

#[test]
fn test_media_missing_token() {
    assert!(parse_media("video 9 UDP/TLS/RTP/SAVPF").is_err());
}

#[test]
fn test_media_invalid_port_number() {
    assert!(parse_media("video 75123 UDP/TLS/RTP/SAVPF 8").is_err());
}

#[test]
fn test_media_invalid_type() {
    assert!(parse_media("invalid 9 UDP/TLS/RTP/SAVPF 8").is_err());
}

#[test]
fn test_media_invalid_transport() {
    assert!(parse_media("audio 9 invalid/invalid 8").is_err());
}

#[test]
fn test_media_invalid_payload() {
    assert!(parse_media("audio 9 UDP/TLS/RTP/SAVPF 300").is_err());
}

fn parse_attribute(value: &str) -> Result<SdpLine, SdpParserResult> {
    let name: &str;
    let mut val: &str = "";
    if value.find(':') == None {
        name = value;
    } else {
        let v: Vec<&str> = value.splitn(2, ':').collect();
        name = v[0];
        val = v[1];
    }
    let attrtype = match name.to_lowercase().as_ref() {
        // TODO TODO TODO
        "candidate" => SdpAttributeType::Candidate,
        "end-of-candidates" => SdpAttributeType::EndOfCandidates,
        "extmap" => SdpAttributeType::Extmap,
        "fingerprint" => SdpAttributeType::Fingerprint,
        "fmtp" => SdpAttributeType::Fmtp,
        "group" => SdpAttributeType::Group,
        "ice-options" => SdpAttributeType::IceOptions,
        "ice-pwd" => SdpAttributeType::IcePwd,
        "ice-ufrag" => SdpAttributeType::IceUfrag,
        "inactive" => SdpAttributeType::Inactive,
        "mid" => SdpAttributeType::Mid,
        "msid" => SdpAttributeType::Msid,
        "msid-semantic" => SdpAttributeType::MsidSemantic,
        "rid" => SdpAttributeType::Rid,
        "recvonly" => SdpAttributeType::Recvonly,
        "rtcp" => SdpAttributeType::Rtcp,
        "rtcp-fb" => SdpAttributeType::RtcpFb,
        "rtcp-mux" => SdpAttributeType::RtcpMux,
        "rtcp-rsize" => SdpAttributeType::RtcpRsize,
        "rtpmap" => SdpAttributeType::Rtpmap,
        "sctpmap" => SdpAttributeType::Sctpmap,
        "sctp-port" => SdpAttributeType::SctpPort,
        "sendonly" => SdpAttributeType::Sendonly,
        "sendrecv" => SdpAttributeType::Sendrecv,
        "setup" => SdpAttributeType::Setup,
        "simulcast" => SdpAttributeType::Simulcast,
        "ssrc" => SdpAttributeType::Ssrc,
        "ssrc-group" => SdpAttributeType::SsrcGroup,
        _ => return Err(SdpParserResult::ParserUnsupported {
              message: "unsupported attribute value".to_string(),
              line: name.to_string() }),
    };
    let mut attr = SdpAttribute::new(attrtype);
    try!(attr.parse_value(val.trim()));
    /*
    println!("attribute: {}, {}", 
             a.name, a.value.some());
             */
    Ok(SdpLine::Attribute { value: attr })
}

#[test]
fn test_parse_attribute_candidate() {
    assert!(parse_attribute("candidate:0 1 UDP 2122252543 172.16.156.106 49760 typ host").is_ok())
}

#[test]
fn test_parse_attribute_end_of_candidates() {
    assert!(parse_attribute("end-of-candidates").is_ok())
}

#[test]
fn test_parse_attribute_extmap() {
    assert!(parse_attribute("extmap:1/sendonly urn:ietf:params:rtp-hdrext:ssrc-audio-level").is_ok())
}

#[test]
fn test_parse_attribute_fingerprint() {
    assert!(parse_attribute("fingerprint:sha-256 CD:34:D1:62:16:95:7B:B7:EB:74:E2:39:27:97:EB:0B:23:73:AC:BC:BF:2F:E3:91:CB:57:A9:9D:4A:A2:0B:40").is_ok())
}

#[test]
fn test_parse_attribute_fmtp() {
    assert!(parse_attribute("fmtp:109 maxplaybackrate=48000;stereo=1;useinbandfec=1").is_ok())
}

#[test]
fn test_parse_attribute_group() {
    assert!(parse_attribute("group:BUNDLE sdparta_0 sdparta_1 sdparta_2").is_ok())
}

#[test]
fn test_parse_attribute_ice_options() {
    assert!(parse_attribute("ice-options:trickle").is_ok())
}

#[test]
fn test_parse_attribute_ice_pwd() {
    assert!(parse_attribute("ice-pwd:e3baa26dd2fa5030d881d385f1e36cce").is_ok())
}

#[test]
fn test_parse_attribute_ice_ufrag() {
    assert!(parse_attribute("ice-ufrag:58b99ead").is_ok())
}

#[test]
fn test_parse_attribute_inactive() {
    assert!(parse_attribute("inactive").is_ok())
}

#[test]
fn test_parse_attribute_mid() {
    assert!(parse_attribute("mid:sdparta_0").is_ok())
}

#[test]
fn test_parse_attribute_msid() {
    assert!(parse_attribute("msid:{5a990edd-0568-ac40-8d97-310fc33f3411} {218cfa1c-617d-2249-9997-60929ce4c405}").is_ok())
}

#[test]
fn test_parse_attribute_msid_semantics() {
    assert!(parse_attribute("msid-semantic:WMS *").is_ok())
}

#[test]
fn test_parse_attribute_rid() {
    assert!(parse_attribute("rid:foo send").is_ok())
}

#[test]
fn test_parse_attribute_recvonly() {
    assert!(parse_attribute("recvonly").is_ok())
}

#[test]
fn test_parse_attribute_rtcp() {
    assert!(parse_attribute("rtcp:9 IN IP4 0.0.0.0").is_ok())
}

#[test]
fn test_parse_attribute_rtcp_fb() {
    assert!(parse_attribute("rtcp-fb:101 ccm fir").is_ok())
}

#[test]
fn test_parse_attribute_rtcp_mux() {
    assert!(parse_attribute("rtcp-mux").is_ok())
}

#[test]
fn test_parse_attribute_rtcp_rsize() {
    assert!(parse_attribute("rtcp-rsize").is_ok())
}

#[test]
fn test_parse_attribute_rtpmap() {
    assert!(parse_attribute("rtpmap:109 opus/48000/2").is_ok())
}

#[test]
fn test_parse_attribute_sctpmap() {
    assert!(parse_attribute("sctpmap:5000 webrtc-datachannel 256").is_ok())
}

#[test]
fn test_parse_attribute_sctp_port() {
    assert!(parse_attribute("sctp-port:5000").is_ok())
}

#[test]
fn test_parse_attribute_simulcast() {
    assert!(parse_attribute("simulcast: send rid=foo;bar").is_ok())
}

#[test]
fn test_parse_attribute_ssrc() {
    assert!(parse_attribute("ssrc:2655508255 cname:{735484ea-4f6c-f74a-bd66-7425f8476c2e}").is_ok())
}

#[test]
fn test_parse_attribute_ssrc_group() {
    assert!(parse_attribute("ssrc-group:FID 3156517279 2673335628").is_ok())
}

// TODO add missing unit tests

fn parse_sdp_line(line: &str) -> Result<SdpLine, SdpParserResult> {
    if line.find('=') == None {
        return Err(SdpParserResult::ParserLineError {
            message: "missing = character in line".to_string(),
            line: line.to_string() });
    }
    let v: Vec<&str> = line.splitn(2, '=').collect();
    if v.len() < 2 {
        return Err(SdpParserResult::ParserLineError {
            message: "failed to split field and attribute".to_string(),
            line: line.to_string() });
    };
    let name = v[0].trim();
    if name.is_empty() || name.len() > 1 {
        return Err(SdpParserResult::ParserLineError {
            message: "field name empty or too long".to_string(),
            line: line.to_string() });
    };
    let value = v[1].trim();
    if value.len() == 0 {
        return Err(SdpParserResult::ParserLineError {
            message: "attribute value has zero length".to_string(),
            line: line.to_string() });
    }
    match name.to_lowercase().as_ref() {
        "a" => { parse_attribute(value) },
        "b" => { parse_bandwidth(value) },
        "c" => { parse_connection(value) },
        "e" => { parse_email(value) },
        "i" => { parse_information(value) },
        "k" => { parse_key(value) },
        "m" => { parse_media(value) },
        "o" => { parse_origin(value) },
        "p" => { parse_phone(value) },
        "r" => { parse_repeat(value) },
        "s" => { parse_session(value) },
        "t" => { parse_timing(value) },
        "u" => { parse_uri(value) },
        "v" => { parse_version(value) },
        "z" => { parse_zone(value) },
        _   => { return Err(SdpParserResult::ParserLineError {
                    message: "unsupported sdp field".to_string(),
                    line: line.to_string() }) }
    }
}

#[test]
fn test_parse_sdp_line_works() {
    assert!(parse_sdp_line("v=0").is_ok());
}

#[test]
fn test_parse_sdp_line_empty_line() {
    assert!(parse_sdp_line("").is_err());
}

#[test]
fn test_parse_sdp_line_without_equal() {
    assert!(parse_sdp_line("abcd").is_err());
    assert!(parse_sdp_line("ab cd").is_err());
}

#[test]
fn test_parse_sdp_line_empty_value() {
    assert!(parse_sdp_line("v=").is_err());
    assert!(parse_sdp_line("o=").is_err());
    assert!(parse_sdp_line("s=").is_err());
}

#[test]
fn test_parse_sdp_line_empty_name() {
    assert!(parse_sdp_line("=abc").is_err());
}

#[test]
fn test_parse_sdp_line_valid_a_line() {
    assert!(parse_sdp_line("a=rtpmap:8 PCMA/8000").is_ok());
}

#[test]
fn test_parse_sdp_line_invalid_a_line() {
    assert!(parse_sdp_line("a=rtpmap:8 PCMA/8000 1").is_err());
}

// TODO add uni tests here
fn parse_media_vector(lines: &[SdpLine]) -> Result<Vec<SdpMedia>, SdpParserResult> {
    let mut media_sections: Vec<SdpMedia> = Vec::new();
    let mut sdp_media = match lines[0] {
        SdpLine::Media{value: ref v} => {SdpMedia::new(v.clone())},
        _ => return Err(SdpParserResult::ParserSequence {
            message: "first line in media section needs to be a media line".to_string(),
            line: None })
    };
    for line in lines.iter().skip(1) {
        match *line {
            SdpLine::Information{value: ref v} => {sdp_media.set_information(v.clone())},
            SdpLine::Connection{value: ref v} => {sdp_media.set_connection(v.clone())},
            SdpLine::Bandwidth{value: ref v} => {sdp_media.add_bandwidth(v.clone());},
            SdpLine::Key{value: ref v} => {sdp_media.set_key(v.clone())},
            SdpLine::Attribute{value: ref v} => {sdp_media.add_attribute(v.clone());},
            SdpLine::Media{value: ref v} => {
                media_sections.push(sdp_media);
                sdp_media = SdpMedia::new(v.clone());
            },

            SdpLine::Email{..} | SdpLine::Phone{..} | SdpLine::Origin{..} |
                SdpLine::Repeat{..} | SdpLine::Session{..} |
                SdpLine::Timing{..} | SdpLine::Uri{..} | SdpLine::Version{..} |
                SdpLine::Zone{..} => return Err(
                    SdpParserResult::ParserSequence {
                        message: "invalid type in media section".to_string(),
                        line: None})
        };
    };
    media_sections.push(sdp_media);
    Ok(media_sections)
}

// TODO add unit tests
fn parse_sdp_vector(lines: &Vec<SdpLine>) -> Result<SdpSession, SdpParserResult> {
    if lines.len() < 5 {
        return Err(SdpParserResult::ParserSequence {
            message: "SDP neeeds at least 5 lines".to_string(),
            line: None })
    }

    // TODO are these mataches really the only way to verify the types?
    let version: u64 = match lines[0] {
        SdpLine::Version{value: v} => v,
        _ => return Err(SdpParserResult::ParserSequence {
            message: "first line needs to be version number".to_string(),
            line: None })
    };
    let origin: SdpOrigin = match lines[1] {
        SdpLine::Origin{value: ref v} => v.clone(),
        _ => return Err(SdpParserResult::ParserSequence {
            message: "second line needs to be origin".to_string(),
            line: None })
    };
    let session: String = match lines[2] {
        SdpLine::Session{value: ref v} => v.clone(),
        _ => return Err(SdpParserResult::ParserSequence {
            message: "third line needs to be session".to_string(),
            line: None })
    };
    let mut sdp_session = SdpSession::new(version,
                                          origin,
                                          session);
    for (i, line) in lines.iter().enumerate().skip(3) {
        match *line {
            SdpLine::Attribute{value: ref v} => {sdp_session.add_attribute(v.clone())},
            SdpLine::Bandwidth{value: ref v} => {sdp_session.add_bandwidth(v.clone())},
            SdpLine::Timing{value: ref v} => {sdp_session.set_timing(v.clone())},
            SdpLine::Media{..} => {sdp_session.extend_media(
                                        try!(parse_media_vector(&lines[i..])))
                                  },
            SdpLine::Origin{..} |
            SdpLine::Session{..} |
            SdpLine::Version{..} => return Err(SdpParserResult::ParserSequence {
                                                        message: "internal parser error".to_string(),
                                                        line: Some(i)}),
            // TODO does anyone really ever need these?
            SdpLine::Connection{..} | SdpLine::Email{..} |
            SdpLine::Information{..} | SdpLine::Key{..} |
            SdpLine::Phone{..} | SdpLine::Repeat{..} |
            SdpLine::Uri{..} | SdpLine::Zone{..}         => (),
        };
        if sdp_session.has_media() {
            break;
        };
    }
    if !sdp_session.has_timing() {
        return Err(SdpParserResult::ParserSequence {
            message: "Missing timing".to_string(),
            line: None},);
    }
    if !sdp_session.has_media() {
        return Err(SdpParserResult::ParserSequence {
            message: "Missing media".to_string(),
            line: None},);
    }
    Ok(sdp_session)
}

pub fn parse_sdp(sdp: &str, fail_on_warning: bool) -> Result<SdpSession, SdpParserResult> {
    if sdp.is_empty() {
        return Err(SdpParserResult::ParserLineError{message: "empty SDP".to_string(),
                                                            line: "".to_string()});
    }
    if sdp.len() < 62 {
        return Err(SdpParserResult::ParserLineError{message: "string to short to be valid SDP".to_string(),
                                                            line: sdp.to_string()});
    }
    let lines = sdp.lines();
    let mut errors: Vec<SdpParserResult> = Vec::new();
    let mut warnings: Vec<SdpParserResult> = Vec::new();
    let mut sdp_lines: Vec<SdpLine> = Vec::new();
    for line in lines {
        match parse_sdp_line(line) {
            Ok(n) => { sdp_lines.push(n); },
            Err(e) => {
                match e {
                    // FIXME is this really a good way to accomplish this?
                    SdpParserResult::ParserLineError { message: x, line: y } =>
                        { errors.push(SdpParserResult::ParserLineError { message: x, line: y}) },
                    SdpParserResult::ParserUnsupported { message: x, line: y } =>
                        {
                            println!("Warning unsupported value encountered: {}\n in line {}", x, y);
                            warnings.push(SdpParserResult::ParserUnsupported { message: x, line: y});
                        },
                    SdpParserResult::ParserSequence {message: x, line: y} =>
                        { errors.push(SdpParserResult::ParserSequence { message: x, line: y})}
                }
            }
        };
    };
    for warning in warnings {
        if fail_on_warning {
            return Err(warning);
        } else {
            match warning {
                SdpParserResult::ParserUnsupported { message: msg, line: l} =>
                    { println!("Parser unknown: {}\n  in line: {}", msg, l) },
                _ => panic!(),
            };
        };
    };
    for error in errors {
        match error {
            SdpParserResult::ParserLineError { message: msg, line: l} =>
                { println!("Parser error: {}\n  in line: {}", msg, l) },
            SdpParserResult::ParserSequence { message: msg, ..} =>
                { println!("Parser sequence: {}", msg)}
            _ => panic!(),
        };
    };
    let session = try!(parse_sdp_vector(&sdp_lines));
    Ok(session)
}

#[test]
fn test_parse_sdp_zero_length_string_fails() {
    assert!(parse_sdp("", true).is_err());
}

#[test]
fn test_parse_sdp_to_short_string() {
    assert!(parse_sdp("fooooobarrrr", true).is_err());
}
