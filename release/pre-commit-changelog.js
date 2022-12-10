// Used for release to update Cargo.toml and Cargo.lock with released version

exports.preCommit = (props) => {
  const fs = require('fs')
  const regex = /(name = "novops"\nversion = ")(.*)(".*)/g;
  
  const cargoContent = fs.readFileSync('Cargo.toml', {encoding:'utf8', flag:'r'});
  const cargoNew = cargoContent.replace(regex, '$1' + props.version + '$3');
  fs.writeFileSync('Cargo.toml', cargoNew, {encoding: 'utf8'})

  const lockContent = fs.readFileSync('Cargo.lock', {encoding:'utf8', flag:'r'});
  const lockNew = lockContent.replace(regex, '$1' + props.version + '$3');
  fs.writeFileSync('Cargo.lock', lockNew, {encoding: 'utf8'})
}