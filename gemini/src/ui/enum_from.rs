/// 组件标识符枚举
#[derive(Clone, PartialEq, Eq)]
pub enum InputIdentifier {
    Model,
    Key,
    SystemInstruction,
    ResponseMineType,
    MaxOutputTokens,
    Temperature,
    TopP,
    TopK,
}

impl TryFrom<i32> for InputIdentifier {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == InputIdentifier::Model as i32 => Ok(InputIdentifier::Model),
            x if x == InputIdentifier::Key as i32 => Ok(InputIdentifier::Key),
            x if x == InputIdentifier::SystemInstruction as i32 => Ok(InputIdentifier::SystemInstruction),
            x if x == InputIdentifier::ResponseMineType as i32 => Ok(InputIdentifier::ResponseMineType),
            x if x == InputIdentifier::MaxOutputTokens as i32 => Ok(InputIdentifier::MaxOutputTokens),
            x if x == InputIdentifier::Temperature as i32 => Ok(InputIdentifier::Temperature),
            x if x == InputIdentifier::TopP as i32 => Ok(InputIdentifier::TopP),
            x if x == InputIdentifier::TopK as i32 => Ok(InputIdentifier::TopK),
            _ => Err(()),
        }
    }
}
