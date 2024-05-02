use std::path::PathBuf;
use convert_case::{Case, Casing};
use async_trait::async_trait;
use serde::Deserialize;
use anyhow;
use sha2::{Sha256, Digest};
use schemars::JsonSchema;
use log::warn;

use crate::core::{ResolveTo, NovopsContext, BytesResolvableInput};
use crate::modules::variables::VariableOutput;


/// 
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct FileInput {
    /// File name to use when auto-generating file and variable name.
    /// if not set, the YAML key for file will be used
    pub name: Option<String>,

    /// DEPRECATED: `dest` is insecure as generated file may be written in insecure directory and/or persist on disk. 
    /// Use `symlink` instead to create a symbolic link pointing to generated file in secure Novops secure directory.
    /// 
    /// Destination where file will be generated. Default to secure Novops working directory.
    /// 
    /// Setting this value may prevent file from being auto-deleted as it won't be managed in a safe location and may remain indefinitely.
    pub dest: Option<String>,

    /// Creates a symbolic link pointing to generated file. If a file already exists
    /// 
    /// Concrete file is still generated in secure Novops working directory, created symlink
    /// will point to concrete file. 
    /// 
    /// For example, `symlink: "./mytoken"` will create a symlink at "./mytoken" which can be used to read 
    /// file directly.
    /// 
    /// If a file already exists at symlink's destination and is NOT a symlink, Novops will fail. 
    /// 
    /// See also `variable` to generate an environment variable pointing to file in secure Novops working directory.
    pub symlink: Option<String>,
    
    /// Environment variable name pointing to generated file. 
    /// 
    /// Example: setting `NPM_TOKEN` will output an environment variable pointing to file path such as 
    /// 
    /// `NPM_TOKEN: /run/user/1000/novops/dev/file_xxx`
    /// 
    /// See also `symlink` to create a symlink pointing to file in secure Novops working directory;
    pub variable: Option<String>,
    
    /// File content 
    pub content: BytesResolvableInput
}


/// Output for FileInput, with final dest, variable and content
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub struct FileOutput {
    pub dest: PathBuf,
    pub symlink: Option<PathBuf>,
    pub variable: VariableOutput,
    pub content: Vec<u8> // TODO buffer? content may be long
}


/// Resolve a FileInput to its FileOutput for given context
#[async_trait]
impl ResolveTo<FileOutput> for FileInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<FileOutput, anyhow::Error> {
        
        // enforce either name or variable as name is used to auto-generate variable 
        // otherwise we can't affect a deterministic variable name from config
        if self.symlink.is_none() &&  self.variable.is_none() && self.dest.is_none(){
            return Err(anyhow::anyhow!("You must specify at least one of `symlink`, `variable` or `dest` for File input {:?}.", self));
        }
        
        let content = match self.content.resolve(ctx).await {
            Ok(c) => c,
            Err(e) => return Err(e),
        };

        // use content hash as file name to ensure the same content generates same file name
        // useful for testing to know where to look for file
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let fname = format!("{:x}", hasher.finalize());

        // if dest provided, use it
        // otherwise use working directory and file name
        let dest = match &self.dest {
            Some(s) => {
                warn!("File input `dest` is deprecated (potentially insecure). Use `symlink` instead. See https://novops.dev/config/files-variables.html");
                PathBuf::from(&s)
            }
            None => ctx.workdir.join(format!("file_{:}", &fname))
        };

        let symlink = self.symlink.clone().map(|sl| PathBuf::from(&sl));

        // variable pointing to file path
        // if variable name is provided, use it
        // otherwise default to NOVOPS_<env>_<key>
        let variable_name = match &self.variable {
            Some(v) => v.clone(),
            None => format!("NOVOPS_{:}_FILE_{:}", 
                &ctx.app_name.to_case(Case::Snake).to_uppercase(), 
                fname.to_case(Case::Snake).to_uppercase()),
        };

        let file_path_str = dest.clone().into_os_string().into_string()
            .map_err(|o| anyhow::anyhow!("Couldn't convert OsString '{:?}' to String", o))?;

        return Ok(
            FileOutput {
                dest:  PathBuf::from(&dest),
                variable: VariableOutput {
                    name: variable_name,
                    value: file_path_str
                },
                symlink,
                content
            }
        )

    }
}