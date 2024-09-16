#[derive(Clone, PartialEq, Eq)]
pub enum AllSettingComponents {
    Model,
    Key,
    SystemInstruction,
    ResponseMineType,
    MaxOutputTokens,
    Temperature,
    TopP,
    TopK,
}

impl TryFrom<i32> for AllSettingComponents {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == AllSettingComponents::Model as i32 => Ok(AllSettingComponents::Model),
            x if x == AllSettingComponents::Key as i32 => Ok(AllSettingComponents::Key),
            x if x == AllSettingComponents::SystemInstruction as i32 => Ok(AllSettingComponents::SystemInstruction),
            x if x == AllSettingComponents::ResponseMineType as i32 => Ok(AllSettingComponents::ResponseMineType),
            x if x == AllSettingComponents::MaxOutputTokens as i32 => Ok(AllSettingComponents::MaxOutputTokens),
            x if x == AllSettingComponents::Temperature as i32 => Ok(AllSettingComponents::Temperature),
            x if x == AllSettingComponents::TopP as i32 => Ok(AllSettingComponents::TopP),
            x if x == AllSettingComponents::TopK as i32 => Ok(AllSettingComponents::TopK),
            _ => Err(()),
        }
    }
}
