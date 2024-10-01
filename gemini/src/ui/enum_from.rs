use super::{component::popup::delete_popup::ButtonType, setting::InputIdentifier, MainFocusComponent};

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

impl TryFrom<i32> for MainFocusComponent {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == MainFocusComponent::InputField as i32 => Ok(MainFocusComponent::InputField),
            x if x == MainFocusComponent::NewChatButton as i32 => Ok(MainFocusComponent::NewChatButton),
            x if x == MainFocusComponent::ChatItemList as i32 => Ok(MainFocusComponent::ChatItemList),
            x if x == MainFocusComponent::SettingButton as i32 => Ok(MainFocusComponent::SettingButton),
            x if x == MainFocusComponent::ChatShow as i32 => Ok(MainFocusComponent::ChatShow),
            _ => Err(()),
        }
    }
}

impl TryFrom<i32> for ButtonType {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == ButtonType::Confirm as i32 => Ok(ButtonType::Confirm),
            x if x == ButtonType::Cancel as i32 => Ok(ButtonType::Cancel),
            _ => Err(()),
        }
    }
}
