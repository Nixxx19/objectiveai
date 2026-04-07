/// Remote source including Mock.
#[derive(Clone, Debug, clap::ValueEnum)]
pub enum Remote {
    Github,
    Filesystem,
    Mock,
}

impl Remote {
    pub fn into_path(self, owner: Option<String>, repository: Option<String>, name: Option<String>, commit: Option<String>) -> Option<objectiveai::RemotePathCommitOptional> {
        match self {
            Remote::Github => {
                Some(objectiveai::RemotePathCommitOptional::Github {
                    owner: owner?,
                    repository: repository?,
                    commit,
                })
            }
            Remote::Filesystem => {
                Some(objectiveai::RemotePathCommitOptional::Filesystem {
                    owner: owner?,
                    repository: repository?,
                    commit,
                })
            }
            Remote::Mock => {
                Some(objectiveai::RemotePathCommitOptional::Mock {
                    name: name?,
                })
            }
        }
    }
}

