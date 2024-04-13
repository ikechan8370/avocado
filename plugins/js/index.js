import { Event } from 'def'
/**
 * event
 * @type {Event}
 */
global.e = {}

/**
 * @type {Logger}
 */
global.logger = {};
/**
 *
 * @type {CheckFunction}
 */
global.check = (a, b) => {
    return a.test(b)
}