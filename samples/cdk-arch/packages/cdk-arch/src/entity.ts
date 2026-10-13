import { Construct } from 'constructs';
import { Architecture } from './architecture';

/**
 * Marker architecture: whatever is scoped under it lives outside the solution
 * (the people and systems the solution interacts with but does not contain).
 */
export const external = new Architecture('external');

export type EntityKind = 'person' | 'system';

export interface EntityUsage {
  target: Construct;
  label?: string;
}

/**
 * Something the code cannot show by itself: a person or a system that uses (or is used by) the solution.
 * Has no runtime part. Declared under `external` for outsiders, or under a regular Architecture for insiders.
 */
export class Entity extends Construct {
  private readonly _usages: EntityUsage[] = [];

  constructor(scope: Construct, id: string, public readonly kind: EntityKind) {
    super(scope, id);
  }

  /**
   * Declare that this entity uses the target, optionally describing how ("says hello").
   */
  uses(target: Construct, label?: string): this {
    this._usages.push({ target, label });
    return this;
  }

  get usages(): readonly EntityUsage[] {
    return this._usages;
  }

  get isExternal(): boolean {
    return this.node.scopes.includes(external);
  }
}
