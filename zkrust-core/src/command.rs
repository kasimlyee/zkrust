//! ZKTeco protocol command definitions

use std::fmt;

use crate::error::{Error, Result};

/// Protocol command codes
///
/// All commands from the ZKTeco Communication Protocol Manual.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum Command {
    // Connection commands
    Connect = 1000,
    Exit = 1001,
    EnableDevice = 1002,
    DisableDevice = 1003,
    Restart = 1004,
    PowerOff = 1005,
    Sleep = 1006,
    Resume = 1007,
    
    // Device interaction
    CaptureFinger = 1009,
    TestTemp = 1011,
    CaptureImage = 1012,
    RefreshData = 1013,
    RefreshOption = 1014,
    TestVoice = 1017,
    
    // Device information
    GetVersion = 1100,
    ChangeSpeed = 1101,
    Auth = 1102,
    
    // Data transfer
    PrepareData = 1500,
    Data = 1501,
    FreeData = 1502,
    
    // Database operations
    DbRrq = 7,
    UserWrq = 8,
    UserTempRrq = 9,
    UserTempWrq = 10,
    OptionsRrq = 11,
    OptionsWrq = 12,
    AttLogRrq = 13,
    ClearData = 14,
    ClearAttLog = 15,
    DeleteUser = 18,
    DeleteUserTemp = 19,
    ClearAdmin = 20,
    
    // Group & timezone management
    UserGrpRrq = 21,
    UserGrpWrq = 22,
    UserTzRrq = 23,
    UserTzWrq = 24,
    GrpTzRrq = 25,
    GrpTzWrq = 26,
    TzRrq = 27,
    TzWrq = 28,
    UlgRrq = 29,
    UlgWrq = 30,
    Unlock = 31,
    ClearAcc = 32,
    ClearOpLog = 33,
    OpLogRrq = 34,
    
    // Device status
    GetFreeSizes = 50,
    EnableClock = 57,
    StartVerify = 60,
    StartEnroll = 61,
    CancelCapture = 62,
    StateRrq = 64,
    WriteLcd = 66,
    ClearLcd = 67,
    GetPinWidth = 69,
    
    // SMS operations
    SmsWrq = 70,
    SmsRrq = 71,
    DeleteSms = 72,
    UDataWrq = 73,
    DeleteUData = 74,
    
    // Access control
    DoorStateRrq = 75,
    WriteMifare = 76,
    EmptyMifare = 78,
    
    // Time operations
    GetTime = 201,
    SetTime = 202,
    
    // Real-time events
    RegEvent = 500,
    
    // Response commands (from device)
    AckOk = 2000,
    AckError = 2001,
    AckData = 2002,
    AckRetry = 2003,
    AckRepeat = 2004,
    AckUnauth = 2005,
    AckUnknown = 0xFFFF,
    AckErrorCmd = 0xFFFD,
    AckErrorInit = 0xFFFC,
    AckErrorData = 0xFFFB,
}

impl Command {
    /// Check if this is a request command (from PC to device)
    pub fn is_request(self) -> bool {
        !self.is_response()
    }
    
    /// Check if this is a response command (from device to PC)
    pub fn is_response(self) -> bool {
        matches!(
            self,
            Self::AckOk
                | Self::AckError
                | Self::AckData
                | Self::AckRetry
                | Self::AckRepeat
                | Self::AckUnauth
                | Self::AckUnknown
                | Self::AckErrorCmd
                | Self::AckErrorInit
                | Self::AckErrorData
        )
    }
    
    /// Check if this is a success response
    pub fn is_success(self) -> bool {
        matches!(self, Self::AckOk | Self::AckData)
    }
    
    /// Check if this is an error response
    pub fn is_error(self) -> bool {
        matches!(
            self,
            Self::AckError
                | Self::AckErrorCmd
                | Self::AckErrorInit
                | Self::AckErrorData
        )
    }
    
    /// Get command name
    pub fn name(self) -> &'static str {
        match self {
            Self::Connect => "CMD_CONNECT",
            Self::Exit => "CMD_EXIT",
            Self::EnableDevice => "CMD_ENABLEDEVICE",
            Self::DisableDevice => "CMD_DISABLEDEVICE",
            Self::Restart => "CMD_RESTART",
            Self::PowerOff => "CMD_POWEROFF",
            Self::Sleep => "CMD_SLEEP",
            Self::Resume => "CMD_RESUME",
            Self::CaptureFinger => "CMD_CAPTUREFINGER",
            Self::TestTemp => "CMD_TEST_TEMP",
            Self::CaptureImage => "CMD_CAPTUREIMAGE",
            Self::RefreshData => "CMD_REFRESHDATA",
            Self::RefreshOption => "CMD_REFRESHOPTION",
            Self::TestVoice => "CMD_TESTVOICE",
            Self::GetVersion => "CMD_GET_VERSION",
            Self::ChangeSpeed => "CMD_CHANGE_SPEED",
            Self::Auth => "CMD_AUTH",
            Self::PrepareData => "CMD_PREPARE_DATA",
            Self::Data => "CMD_DATA",
            Self::FreeData => "CMD_FREE_DATA",
            Self::DbRrq => "CMD_DB_RRQ",
            Self::UserWrq => "CMD_USER_WRQ",
            Self::UserTempRrq => "CMD_USERTEMP_RRQ",
            Self::UserTempWrq => "CMD_USERTEMP_WRQ",
            Self::OptionsRrq => "CMD_OPTIONS_RRQ",
            Self::OptionsWrq => "CMD_OPTIONS_WRQ",
            Self::AttLogRrq => "CMD_ATTLOG_RRQ",
            Self::ClearData => "CMD_CLEAR_DATA",
            Self::ClearAttLog => "CMD_CLEAR_ATTLOG",
            Self::DeleteUser => "CMD_DELETE_USER",
            Self::DeleteUserTemp => "CMD_DELETE_USERTEMP",
            Self::ClearAdmin => "CMD_CLEAR_ADMIN",
            Self::GetTime => "CMD_GET_TIME",
            Self::SetTime => "CMD_SET_TIME",
            Self::RegEvent => "CMD_REG_EVENT",
            Self::AckOk => "CMD_ACK_OK",
            Self::AckError => "CMD_ACK_ERROR",
            Self::AckData => "CMD_ACK_DATA",
            Self::AckUnauth => "CMD_ACK_UNAUTH",
            _ => "CMD_UNKNOWN",
        }
    }
}

impl From<Command> for u16 {
    fn from(cmd: Command) -> u16 {
        cmd as u16
    }
}

impl TryFrom<u16> for Command {
    type Error = Error;
    
    fn try_from(value: u16) -> Result<Self> {
        match value {
            1000 => Ok(Self::Connect),
            1001 => Ok(Self::Exit),
            1002 => Ok(Self::EnableDevice),
            1003 => Ok(Self::DisableDevice),
            1004 => Ok(Self::Restart),
            1005 => Ok(Self::PowerOff),
            1006 => Ok(Self::Sleep),
            1007 => Ok(Self::Resume),
            1009 => Ok(Self::CaptureFinger),
            1011 => Ok(Self::TestTemp),
            1012 => Ok(Self::CaptureImage),
            1013 => Ok(Self::RefreshData),
            1014 => Ok(Self::RefreshOption),
            1017 => Ok(Self::TestVoice),
            1100 => Ok(Self::GetVersion),
            1101 => Ok(Self::ChangeSpeed),
            1102 => Ok(Self::Auth),
            1500 => Ok(Self::PrepareData),
            1501 => Ok(Self::Data),
            1502 => Ok(Self::FreeData),
            7 => Ok(Self::DbRrq),
            8 => Ok(Self::UserWrq),
            9 => Ok(Self::UserTempRrq),
            10 => Ok(Self::UserTempWrq),
            11 => Ok(Self::OptionsRrq),
            12 => Ok(Self::OptionsWrq),
            13 => Ok(Self::AttLogRrq),
            14 => Ok(Self::ClearData),
            15 => Ok(Self::ClearAttLog),
            18 => Ok(Self::DeleteUser),
            19 => Ok(Self::DeleteUserTemp),
            20 => Ok(Self::ClearAdmin),
            21 => Ok(Self::UserGrpRrq),
            22 => Ok(Self::UserGrpWrq),
            23 => Ok(Self::UserTzRrq),
            24 => Ok(Self::UserTzWrq),
            25 => Ok(Self::GrpTzRrq),
            26 => Ok(Self::GrpTzWrq),
            27 => Ok(Self::TzRrq),
            28 => Ok(Self::TzWrq),
            29 => Ok(Self::UlgRrq),
            30 => Ok(Self::UlgWrq),
            31 => Ok(Self::Unlock),
            32 => Ok(Self::ClearAcc),
            33 => Ok(Self::ClearOpLog),
            34 => Ok(Self::OpLogRrq),
            50 => Ok(Self::GetFreeSizes),
            57 => Ok(Self::EnableClock),
            60 => Ok(Self::StartVerify),
            61 => Ok(Self::StartEnroll),
            62 => Ok(Self::CancelCapture),
            64 => Ok(Self::StateRrq),
            66 => Ok(Self::WriteLcd),
            67 => Ok(Self::ClearLcd),
            69 => Ok(Self::GetPinWidth),
            70 => Ok(Self::SmsWrq),
            71 => Ok(Self::SmsRrq),
            72 => Ok(Self::DeleteSms),
            73 => Ok(Self::UDataWrq),
            74 => Ok(Self::DeleteUData),
            75 => Ok(Self::DoorStateRrq),
            76 => Ok(Self::WriteMifare),
            78 => Ok(Self::EmptyMifare),
            201 => Ok(Self::GetTime),
            202 => Ok(Self::SetTime),
            500 => Ok(Self::RegEvent),
            2000 => Ok(Self::AckOk),
            2001 => Ok(Self::AckError),
            2002 => Ok(Self::AckData),
            2003 => Ok(Self::AckRetry),
            2004 => Ok(Self::AckRepeat),
            2005 => Ok(Self::AckUnauth),
            0xFFFF => Ok(Self::AckUnknown),
            0xFFFD => Ok(Self::AckErrorCmd),
            0xFFFC => Ok(Self::AckErrorInit),
            0xFFFB => Ok(Self::AckErrorData),
            _ => Err(Error::UnknownCommand(value)),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.name(), *self as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_conversion() {
        assert_eq!(u16::from(Command::Connect), 1000);
        assert_eq!(Command::try_from(1000).unwrap(), Command::Connect);
    }
    
    #[test]
    fn test_command_is_response() {
        assert!(Command::AckOk.is_response());
        assert!(!Command::Connect.is_response());
    }
    
    #[test]
    fn test_command_is_success() {
        assert!(Command::AckOk.is_success());
        assert!(Command::AckData.is_success());
        assert!(!Command::AckError.is_success());
    }
    
    #[test]
    fn test_unknown_command() {
        let result = Command::try_from(9999);
        assert!(result.is_err());
    }
}