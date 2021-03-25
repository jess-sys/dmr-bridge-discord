const chalk = require('chalk');

function error(entity, status, message) {
    let e_message = "";
    if (message !== undefined) {
        e_message = ' : ' + message
    }
    console.log(`[${entity}][${chalk.red(status)}]${e_message}`)
}

function warn(entity, status, message) {
    let e_message = "";
    if (message !== undefined) {
        e_message = ' : ' + message
    }
    console.log(`[${entity}][${chalk.yellow(status)}]${e_message}`)
}

function info(entity, status, message) {
    let e_message = "";
    if (message !== undefined) {
        e_message = ' : ' + message
    }
    console.log(`[${entity}][${chalk.blue(status)}]${e_message}`)
}

function success(entity, status, message) {
    let e_message = "";
    if (message !== undefined) {
        e_message = ' : ' + message
    }
    console.log(`[${entity}][${chalk.green(status)}]${e_message}`)
}

module.exports = {
    success,
    info,
    warn,
    error
}