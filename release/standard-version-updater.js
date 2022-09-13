// Used by standard-release to update Cargo.toml and Cargo.lock with released version

module.exports.readVersion = function (contents) {

  console.log("Reading Novops version of Cargo.[toml|lock]")

  const regexp = /name = "novops"\nversion = "(.*)"/g;
  const array =  [ ...contents.matchAll(regexp)];

  console.log(`Regex results for Cargo.[toml|lock] ${array}`)
  
  return array[0][1];
}

module.exports.writeVersion = function (contents, version) {
  const regex = /(name = "novops"\nversion = ")(.*)(".*)/g;
  return contents.replace(regex, '$1' + version + '$3');
}