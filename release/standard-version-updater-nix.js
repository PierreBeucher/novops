// Used by standard-release to update Cargo.nix with released version

module.exports.readVersion = function (contents) {

  console.log("Reading Novops version of Cargo.nix")

  const regexp = /novops = rustPackages\.unknown\.novops\."(.*)"/g;
  const array =  [ ...contents.matchAll(regexp)];

  console.log(`Regex results for Cargo.nix ${array}`)
  
  return array[0][1];
}

// Replace references of Novops version by newly released version
module.exports.writeVersion = function (contents, version) {
  const regexPackage = /(novops \= rustPackages\.unknown\.novops\.")(.*)(".*)/g;
  const regexVersion = /(name = "novops".*\n.*version = ")(.*)(".*)/g;
  const regexOverride = /("unknown"\.novops\.")(.*)(".)/g

  return contents.replace(regexPackage, '$1' + version + '$3')
    .replace(regexOverride, '$1' + version + '$3')
    .replace(regexVersion, '$1' + version + '$3');
}