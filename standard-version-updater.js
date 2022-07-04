// standard-version-updater.js

module.exports.readVersion = function (contents) {
  const regexp = /name = "novops"\nversion = "(.*)"/g;
  const array =  [ ...contents.matchAll(regexp)];

  return array[0][1];
}

module.exports.writeVersion = function (contents, version) {
  const regex = /(name = "novops"\nversion = ")(.*)(".*)/g;
  return contents.replace(regex, '$1' + version + '$3');
}