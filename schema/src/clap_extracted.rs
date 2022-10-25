// This module was extracted via `cargo expand` because unfortunately there is no way to specify
// manual bounds on clap derives.
/*
use crate::{Key, Prefix};

impl<Params, SubPrefix> clap::FromArgMatches for Prefix<Params, SubPrefix>
where
    Params: clap::Args,
    SubPrefix: clap::Subcommand,
{
    fn from_arg_matches(
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        Self::from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }
    fn from_arg_matches_mut(
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        #![allow(deprecated)]
        let v = Prefix {
            params: <Params as clap::FromArgMatches>::from_arg_matches_mut(__clap_arg_matches)?,
            child: {
                if __clap_arg_matches
                    .subcommand_name()
                    .map(<SubPrefix as clap::Subcommand>::has_subcommand)
                    .unwrap_or(false)
                {
                    Some(<SubPrefix as clap::FromArgMatches>::from_arg_matches_mut(
                        __clap_arg_matches,
                    )?)
                } else {
                    None
                }
            },
        };
        ::std::result::Result::Ok(v)
    }
    fn update_from_arg_matches(
        &mut self,
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        self.update_from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }
    fn update_from_arg_matches_mut(
        &mut self,
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        #![allow(deprecated)]
        {
            #[allow(non_snake_case)]
            let params = &mut self.params;
            <Params as clap::FromArgMatches>::update_from_arg_matches_mut(
                params,
                __clap_arg_matches,
            )?;
        }
        {
            #[allow(non_snake_case)]
            let child = &mut self.child;
            if let Some(child) = child.as_mut() {
                <SubPrefix as clap::FromArgMatches>::update_from_arg_matches_mut(
                    child,
                    __clap_arg_matches,
                )?;
            } else {
                *child = Some(<SubPrefix as clap::FromArgMatches>::from_arg_matches_mut(
                    __clap_arg_matches,
                )?);
            }
        }
        ::std::result::Result::Ok(())
    }
}

impl<Params, SubPrefix> clap::Args for Prefix<Params, SubPrefix>
where
    Params: clap::Args,
    SubPrefix: clap::Subcommand,
{
    fn group_id() -> Option<clap::Id> {
        Some(clap::Id::from("Prefix"))
    }
    fn augment_args<'b>(__clap_app: clap::Command) -> clap::Command {
        {
            let __clap_app = __clap_app.group(clap::ArgGroup::new("Prefix").multiple(true).args({
                let members: [clap::Id; 0] = [];
                members
            }));
            let __clap_app = __clap_app;
            let __clap_app = <Params as clap::Args>::augment_args(__clap_app);

            <SubPrefix as clap::Subcommand>::augment_subcommands(__clap_app)
        }
    }
    fn augment_args_for_update<'b>(__clap_app: clap::Command) -> clap::Command {
        {
            let __clap_app = __clap_app.group(clap::ArgGroup::new("Prefix").multiple(true).args({
                let members: [clap::Id; 0] = [];
                members
            }));
            let __clap_app = __clap_app;
            let __clap_app = <Params as clap::Args>::augment_args_for_update(__clap_app);
            <SubPrefix as clap::Subcommand>::augment_subcommands(__clap_app)
        }
    }
}

impl<Params, SubKey> clap::FromArgMatches for Key<Params, SubKey>
where
    Params: clap::Args,
    SubKey: clap::Subcommand,
{
    fn from_arg_matches(
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        Self::from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }
    fn from_arg_matches_mut(
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        #![allow(deprecated)]
        let v = Key {
            params: <Params as clap::FromArgMatches>::from_arg_matches_mut(__clap_arg_matches)?,
            child: { <SubKey as clap::FromArgMatches>::from_arg_matches_mut(__clap_arg_matches)? },
        };
        ::std::result::Result::Ok(v)
    }
    fn update_from_arg_matches(
        &mut self,
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        self.update_from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }
    fn update_from_arg_matches_mut(
        &mut self,
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        #![allow(deprecated)]
        {
            #[allow(non_snake_case)]
            let params = &mut self.params;
            <Params as clap::FromArgMatches>::update_from_arg_matches_mut(
                params,
                __clap_arg_matches,
            )?;
        }
        {
            #[allow(non_snake_case)]
            let child = &mut self.child;
            <SubKey as clap::FromArgMatches>::update_from_arg_matches_mut(
                child,
                __clap_arg_matches,
            )?;
        }
        ::std::result::Result::Ok(())
    }
}

impl<Params, SubKey> clap::Args for Key<Params, SubKey>
where
    Params: clap::Args,
    SubKey: clap::Subcommand,
{
    fn group_id() -> Option<clap::Id> {
        Some(clap::Id::from("Key"))
    }
    fn augment_args<'b>(__clap_app: clap::Command) -> clap::Command {
        {
            let __clap_app = __clap_app.group(clap::ArgGroup::new("Key").multiple(true).args({
                let members: [clap::Id; 0] = [];
                members
            }));
            let __clap_app = __clap_app;
            let __clap_app = <Params as clap::Args>::augment_args(__clap_app);
            let __clap_app = <SubKey as clap::Subcommand>::augment_subcommands(__clap_app);
            __clap_app
                .subcommand_required(true)
                .arg_required_else_help(true)
        }
    }
    fn augment_args_for_update<'b>(__clap_app: clap::Command) -> clap::Command {
        {
            let __clap_app = __clap_app.group(clap::ArgGroup::new("Key").multiple(true).args({
                let members: [clap::Id; 0] = [];
                members
            }));
            let __clap_app = __clap_app;
            let __clap_app = <Params as clap::Args>::augment_args_for_update(__clap_app);
            <SubKey as clap::Subcommand>::augment_subcommands(__clap_app)
        }
    }
}
*/
