const chalk = require('chalk');

export function error(entity, status, message?) {
    let e_message = "";
    if (message !== undefined) {
        e_message = ' : ' + message
    }
    console.log(`[${entity}][${chalk.red(status)}]${e_message}`)
}

export function warn(entity, status, message?) {
    let e_message = "";
    if (message !== undefined) {
        e_message = ' : ' + message
    }
    console.log(`[${entity}][${chalk.yellow(status)}]${e_message}`)
}

export function info(entity, status, message?) {
    let e_message = "";
    if (message !== undefined) {
        e_message = ' : ' + message
    }
    console.log(`[${entity}][${chalk.blue(status)}]${e_message}`)
}

export function success(entity, status, message?) {
    let e_message = "";
    if (message !== undefined) {
        e_message = ' : ' + message
    }
    console.log(`[${entity}][${chalk.green(status)}]${e_message}`)
}