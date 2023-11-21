use crate::core::app::v1alpha1::AppParameters;
use crate::Name;

impl Name for AppParameters {
    const NAME: &'static str = "AppParameters";
    const PACKAGE: &'static str = "penumbra.core.app.v1alpha1";
}
