/* tslint:disable */
/* eslint-disable */
/**
 * Todo API
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
/**
 * 
 * @export
 * @interface PatchTask
 */
export interface PatchTask {
    /**
     * 
     * @type {string}
     * @memberof PatchTask
     */
    title?: string;
    /**
     * 
     * @type {boolean}
     * @memberof PatchTask
     */
    done?: boolean;
}

/**
 * Check if a given object implements the PatchTask interface.
 */
export function instanceOfPatchTask(value: object): boolean {
    let isInstance = true;

    return isInstance;
}

export function PatchTaskFromJSON(json: any): PatchTask {
    return PatchTaskFromJSONTyped(json, false);
}

export function PatchTaskFromJSONTyped(json: any, ignoreDiscriminator: boolean): PatchTask {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'title': !exists(json, 'title') ? undefined : json['title'],
        'done': !exists(json, 'done') ? undefined : json['done'],
    };
}

export function PatchTaskToJSON(value?: PatchTask | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'title': value.title,
        'done': value.done,
    };
}

