// import { Event } from 'def'
/**
 * event
 * @type {import('def').Event}
 */
global.e = {}

/**
 * @type {import('def').Logger}
 */
global.logger = {};
/**
 *
 * @type {import('def').CheckFunction}
 */
global.check = (a, b) => {
    return a.test(b)
}
/**
 *
 * @type {import('def').AvocadoBot}
 */
global.Bot = {}