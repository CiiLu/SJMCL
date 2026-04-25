import type { MouseEventHandler } from "react";
import type { IconType } from "react-icons";
import { type ExtensionSlotKey, ExtensionUISlotKey } from "@/enums/extension";
import type { InstanceSummary } from "@/models/instance/misc";
import type { WorldInfo } from "@/models/instance/world";
import type { ExtensionContributionBase } from "./contribution";

export interface CommonIconButtonSlotItem {
  icon: string | IconType;
  label?: string;
  onClick?: MouseEventHandler<HTMLButtonElement>;
}

export interface ExtensionSlotContextMap {
  [ExtensionUISlotKey.InstanceWorldItemMenuOperations]: {
    save: WorldInfo;
    instanceId: string | undefined;
    summary: InstanceSummary | undefined;
  };
}

export interface ExtensionSlotItemMap {
  [ExtensionUISlotKey.InstanceWorldItemMenuOperations]: CommonIconButtonSlotItem;
}

export interface ExtensionSlotDefinition<K extends ExtensionSlotKey> {
  getItems: (context: ExtensionSlotContextMap[K]) => ExtensionSlotItemMap[K][];
}

export interface ExtensionSlotContribution<K extends ExtensionSlotKey>
  extends ExtensionSlotDefinition<K>, ExtensionContributionBase {
  key: K;
}

export type ExtensionSlotRegistry = Partial<{
  [K in ExtensionSlotKey]: ExtensionSlotDefinition<K>;
}>;

export type ExtensionSlotContributionRegistry = Partial<{
  [K in ExtensionSlotKey]: ExtensionSlotContribution<K>;
}>;
