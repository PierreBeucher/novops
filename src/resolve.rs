use log::info;
use tokio::task::JoinSet;
use anyhow::Context;

use crate::{
    core::{NovopsContext, NovopsEnvironmentInput, ResolveTo}, 
    modules::{aws::config::AwsInput, files::{FileInput, FileOutput}, hashivault::config::HashiVaultInput, sops::SopsDotenvInput, variables::{VariableInput, VariableOutput}}
};

pub async fn resolve_environment_inputs_parallel(ctx: &NovopsContext, inputs: NovopsEnvironmentInput) 
    -> Result<(Vec<VariableOutput>, Vec<FileOutput>), anyhow::Error>
{
    
    //
    // Resolve all inputs in parallel
    // For each inputs (variables, files, aws, hashivault, etc.)
    // Run a future resolving to a generic (Vec<VariableOutput>, Vec<FileOutput>) type
    // So that each can be run in parallel in a single JoinSet. 
    //

    // Spawn every resolve tasks into JoinSet
    let mut resolve_tasks = JoinSet::new();

    for i in inputs.variables.unwrap_or_default() {
        let var_fut = resolve_and_wrap_variable_input(ctx.clone(), i);
        resolve_tasks.spawn(var_fut);
    };

    for f in inputs.files.unwrap_or_default() {
        let file_fut = resolve_and_wrap_file_input(ctx.clone(), f);
        resolve_tasks.spawn(file_fut);
    };

    let sops = resolve_and_wrap_sops_input(ctx.clone(), inputs.sops_dotenv);
    resolve_tasks.spawn(sops);

    let aws = resolve_and_wrap_aws_input(ctx.clone(), inputs.aws);
    resolve_tasks.spawn(aws);
    
    let hashivault = resolve_and_wrap_hashivault_input(ctx.clone(), inputs.hashivault);
    resolve_tasks.spawn(hashivault);

    // Await on each output result
    let mut output_results = vec![];
    while let Some(res) = resolve_tasks.join_next().await {
    
        // Result is imbricated Result<Result<_, anyhow::Error>, JoinError>
        // Wrap potential JoinError as anyhow::Error
        let wrapped_res = match res {
            Ok(ok) => ok,
            Err(err) => Err(anyhow::anyhow!(err)),
        };

        output_results.push(wrapped_res);
    }

    // Parse all outputs and discriminate ok and errors
    let mut var_outputs = vec![];
    let mut file_outputs = vec![];
    let mut resolve_errors = vec![];
    for result in output_results {
        match result {
            Ok(o) => {
                var_outputs.extend(o.0);
                file_outputs.extend(o.1);
            },
            Err(err) => resolve_errors.push(err),
        }
    }

    if !resolve_errors.is_empty() {
        // Build human-readable error with all existing errors
        let mut final_message = String::from("Failed to resolve one or more Inputs:\n");
        for err in resolve_errors {
            final_message.push_str(format!("\n---\n{:?}\n", err).as_str());
        }
        return Err(anyhow::format_err!(final_message));
    }

    Ok( (var_outputs, file_outputs) )
    
}

async fn resolve_and_wrap_file_input(ctx: NovopsContext, f: FileInput) -> Result<(Vec<VariableOutput>, Vec<FileOutput>), anyhow::Error>  {

    info!("Resolving file input {:?}", &f);

    let result = f.resolve(&ctx).await
        .with_context(|| format!("Couldn't resolve file input {:?}", &f))
        .map(|o| (vec![], vec![o]) )?;

    info!("Resolved file input {:?}", &f);

    Ok(result)
}

async fn resolve_and_wrap_variable_input(ctx: NovopsContext, i: VariableInput) -> Result<(Vec<VariableOutput>, Vec<FileOutput>), anyhow::Error>  {

    info!("Resolving variable input {:}", &i.name);

    let result = i.resolve(&ctx).await
        .with_context(|| format!("Couldn't resolve variable input {:}", &i.name))
        .map(|o| (vec![o], vec![]) )?;

    info!("Resolved variable input {:}", &i.name);

    Ok(result)
}

async fn resolve_and_wrap_sops_input(ctx: NovopsContext, sops_vec_opt: Option<Vec<SopsDotenvInput>>) -> Result<(Vec<VariableOutput>, Vec<FileOutput>), anyhow::Error>  {

    match sops_vec_opt {
        Some(sops_vec) => {
            info!("Resolving SOPS Dotenv inputs");

            let mut result = Vec::new();

            for sops in sops_vec {
                let r = sops.resolve(&ctx).await
                    .with_context(|| format!("Could not resolve SopsDotenv input {:?}", sops))?;

                result.extend(r);
            }

            info!("Resolved SOPS Dotenv inputs");

            Ok( (result, vec![]) )
        },
        None => Ok( (vec![], vec![]) )
    }
}

async fn resolve_and_wrap_hashivault_input(ctx: NovopsContext, hashivault: Option<HashiVaultInput>) -> Result<(Vec<VariableOutput>, Vec<FileOutput>), anyhow::Error> {

    match hashivault {
        Some(hashivault) => {

            info!("Resolving Hashivault inputs");

            let r = hashivault.aws.resolve(&ctx).await
                .with_context(|| format!("Could not resolve Hashivault input {:?}", hashivault))?;

            info!("Resolved Hashivault inputs");

            Ok( (r, vec![]) )
        },

        None => Ok( (vec![], vec![]) ),
    }

    
}

async fn resolve_and_wrap_aws_input(ctx: NovopsContext, aws: Option<AwsInput>) -> Result<(Vec<VariableOutput>, Vec<FileOutput>), anyhow::Error> {

    match aws {
        Some(aws) => {
            info!("Resolving AWS inputs");

            let vars = aws.assume_role.resolve(&ctx).await
                .with_context(|| format!("Could not resolve AWS input {:?}", aws))?;

            info!("Resolved AWS inputs");

            Ok( (vars, vec![]) )
            
        },
        None => Ok( (vec![], vec![]) ),
    }
}