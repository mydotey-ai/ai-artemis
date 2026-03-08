/**
 * Artemis Console - Complete TypeScript Type Definitions
 * Aligns with Artemis Rust backend API specifications
 *
 * This file contains all type definitions needed for the Artemis management console,
 * ensuring complete type safety when interacting with the backend API.
 */

// =====================================================
// 1. COMMON/SHARED TYPES
// =====================================================

/**
 * Response Status Enum
 * Indicates the status of an API request
 */
export const ErrorCode = {
  SUCCESS: 'success',
  BAD_REQUEST: 'bad-request',
  SERVICE_UNAVAILABLE: 'service-unavailable',
  RATE_LIMITED: 'rate-limited',
  INTERNAL_ERROR: 'internal-error',
} as const;

export type ErrorCode = typeof ErrorCode[keyof typeof ErrorCode];

/**
 * Standard Response Status
 * Used in most API responses to indicate success/failure
 */
export interface ResponseStatus {
  errorCode: ErrorCode;
  errorMessage?: string;
}

/**
 * Generic API Response wrapper
 * Used for most REST API endpoints
 */
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message?: string;
}

/**
 * Pagination parameters
 */
export interface PaginationParams {
  limit?: number;
  offset?: number;
}

// =====================================================
// 2. INSTANCE TYPES
// =====================================================

/**
 * Instance Status Enum
 * Represents the health status of a service instance
 */
export const InstanceStatus = {
  STARTING: 'starting',
  UP: 'up',
  DOWN: 'down',
  UNHEALTHY: 'unhealthy',
  UNKNOWN: 'unknown',
} as const;

export type InstanceStatus = typeof InstanceStatus[keyof typeof InstanceStatus];

/**
 * Service Instance
 * Core entity representing a registered service instance
 */
export interface Instance {
  regionId: string;
  zoneId: string;
  groupId?: string;
  serviceId: string;
  instanceId: string;
  machineName?: string;
  ip: string;
  port: number;
  protocol?: string;
  url: string;
  health_check_url?: string;
  status: InstanceStatus;
  metadata?: Record<string, string>;
}

/**
 * Service Instance (alias for Instance)
 * Core entity representing a registered service instance
 */
export type ServiceInstance = Instance;

/**
 * Instance Key
 * Unique identifier for an instance
 */
export interface InstanceKey {
  regionId: string;
  zoneId: string;
  serviceId: string;
  groupId: string;
  instanceId: string;
}

// =====================================================
// 3. SERVICE TYPES
// =====================================================

/**
 * Service Group
 * Represents a collection of service instances
 */
export interface ServiceGroup {
  groupId?: number;
  serviceId: string;
  regionId: string;
  zoneId: string;
  name: string;
  group_type: GroupType;
  status: GroupStatus;
  description?: string;
  tags?: GroupTag[];
  metadata?: Record<string, string>;
  created_at?: number;
  updated_at?: number;
}

/**
 * Group Status Enum
 */
export const GroupStatus = {
  ACTIVE: 'active',
  INACTIVE: 'inactive',
} as const;

export type GroupStatus = typeof GroupStatus[keyof typeof GroupStatus];

/**
 * Group Type Enum
 */
export const GroupType = {
  PHYSICAL: 'physical',
  LOGICAL: 'logical',
} as const;

export type GroupType = typeof GroupType[keyof typeof GroupType];

/**
 * Group Tag
 * Key-value metadata for groups
 */
export interface GroupTag {
  key: string;
  value: string;
}

/**
 * Group Instance Association
 * Links an instance to a group
 */
export interface GroupInstance {
  id?: number;
  groupId: number;
  instanceId: string;
  regionId: string;
  zoneId: string;
  serviceId: string;
  binding_type?: BindingType;
  operatorId?: string;
  created_at?: number;
}

/**
 * Binding Type Enum
 */
export const BindingType = {
  MANUAL: 'manual',
  AUTO: 'auto',
} as const;

export type BindingType = typeof BindingType[keyof typeof BindingType];

/**
 * Service
 * Represents a service with its instances and routing info
 */
export interface Service {
  serviceId: string;
  metadata?: Record<string, string>;
  instances: Instance[];
  logic_instances?: Instance[];
  route_rules?: RouteRule[];
}

/**
 * Discovery Configuration
 * Parameters for service discovery requests
 */
export interface DiscoveryConfig {
  serviceId: string;
  regionId: string;
  zoneId: string;
  discovery_data?: Record<string, string>;
}

/**
 * Service Group (alternative definition from route.rs)
 * Represents a service group in the routing context
 */
export interface Group {
  groupId?: number;
  serviceId: string;
  regionId: string;
  zoneId: string;
  name: string;
  appId?: string;
  description?: string;
  status: GroupStatus;
  metadata?: Record<string, string>;
}

// =====================================================
// 4. DISCOVERY REQUEST/RESPONSE TYPES
// =====================================================

/**
 * Get Service Request
 */
export interface GetServiceRequest {
  discovery_config: DiscoveryConfig;
}

/**
 * Get Service Response
 */
export interface GetServiceResponse {
  responseStatus: ResponseStatus;
  service?: Service;
}

/**
 * Get Services Request
 */
export interface GetServicesRequest {
  regionId: string;
  zoneId: string;
}

/**
 * Get Services Response
 */
export interface GetServicesResponse {
  responseStatus: ResponseStatus;
  services: Service[];
}

/**
 * Get Services Delta Request
 * For incremental service discovery
 */
export interface GetServicesDeltaRequest {
  regionId: string;
  zoneId: string;
  since_timestamp: number;
}

/**
 * Get Services Delta Response
 */
export interface GetServicesDeltaResponse {
  responseStatus: ResponseStatus;
  services: Service[];
  current_timestamp: number;
}

/**
 * Lookup Services Request
 * Batch lookup for multiple services
 */
export interface LookupServicesRequest {
  discovery_configs: DiscoveryConfig[];
}

/**
 * Lookup Services Response
 */
export interface LookupServicesResponse {
  responseStatus: ResponseStatus;
  services: Service[];
}

// =====================================================
// 5. REGISTRATION REQUEST/RESPONSE TYPES
// =====================================================

/**
 * Register Request
 */
export interface RegisterRequest {
  instances: Instance[];
}

/**
 * Register Response
 */
export interface RegisterResponse {
  responseStatus: ResponseStatus;
  failed_instances?: Instance[];
}

/**
 * Heartbeat Request
 */
export interface HeartbeatRequest {
  instance_keys: InstanceKey[];
}

/**
 * Heartbeat Response
 */
export interface HeartbeatResponse {
  responseStatus: ResponseStatus;
  failed_instance_keys?: InstanceKey[];
}

/**
 * Unregister Request
 */
export interface UnregisterRequest {
  instance_keys: InstanceKey[];
}

/**
 * Unregister Response
 */
export interface UnregisterResponse {
  responseStatus: ResponseStatus;
}

// =====================================================
// 6. MANAGEMENT - INSTANCE OPERATION TYPES
// =====================================================

/**
 * Instance Operation Enum
 * Operation types for instance management
 */
export const InstanceOperationType = {
  PULL_IN: 'pullin',
  PULL_OUT: 'pullout',
} as const;

export type InstanceOperationType = typeof InstanceOperationType[keyof typeof InstanceOperationType];

/**
 * Instance Operation Record
 */
export interface InstanceOperationRecord {
  instanceKey: InstanceKey;
  operation: InstanceOperationType;
  operationComplete: boolean;
  operatorId: string;
  token?: string;
}

/**
 * Operate Instance Request
 */
export interface OperateInstanceRequest {
  instanceKey: InstanceKey;
  operation: InstanceOperationType;
  operationComplete?: boolean;
  operatorId: string;
  token?: string;
}

/**
 * Operate Instance Response
 */
export interface OperateInstanceResponse {
  status: ResponseStatus;
}

/**
 * Get Instance Operations Request
 */
export interface GetInstanceOperationsRequest {
  instance_key: InstanceKey;
}

/**
 * Get Instance Operations Response
 */
export interface GetInstanceOperationsResponse {
  status: ResponseStatus;
  instanceOperationRecords: InstanceOperationRecord[];
}

/**
 * Is Instance Down Request
 */
export interface IsInstanceDownRequest {
  instance_key: InstanceKey;
}

/**
 * Is Instance Down Response
 */
export interface IsInstanceDownResponse {
  status: ResponseStatus;
  is_down: boolean;
}

// =====================================================
// 7. MANAGEMENT - SERVER OPERATION TYPES
// =====================================================

/**
 * Server Operation Enum
 */
export const ServerOperationType = {
  PULL_IN: 'pullin',
  PULL_OUT: 'pullout',
} as const;

export type ServerOperationType = typeof ServerOperationType[keyof typeof ServerOperationType];

/**
 * Server Operation Record
 */
export interface ServerOperationRecord {
  serverId: string;
  regionId: string;
  operation: ServerOperationType;
  operatorId: string;
  operationTime: number;
}

/**
 * Server Operation Info
 * Used in batch query responses
 */
export interface ServerOperationInfo {
  serverId: string;
  regionId: string;
  operation: ServerOperationType;
}

/**
 * Operate Server Request
 */
export interface OperateServerRequest {
  serverId: string;
  regionId: string;
  operation: ServerOperationType;
  operationComplete?: boolean;
  operatorId: string;
  token?: string;
}

/**
 * Operate Server Response
 */
export interface OperateServerResponse {
  status: ResponseStatus;
}

/**
 * Is Server Down Request
 */
export interface IsServerDownRequest {
  serverId: string;
  regionId: string;
}

/**
 * Is Server Down Response
 */
export interface IsServerDownResponse {
  status: ResponseStatus;
  is_down: boolean;
}

/**
 * Get All Instance Operations Request
 */
export interface GetAllInstanceOperationsRequest {
  regionId?: string;
}

/**
 * Get All Instance Operations Response
 */
export interface GetAllInstanceOperationsResponse {
  status: ResponseStatus;
  instance_operation_records: InstanceOperationRecord[];
}

/**
 * Get All Server Operations Request
 */
export interface GetAllServerOperationsRequest {
  regionId?: string;
}

/**
 * Get All Server Operations Response
 */
export interface GetAllServerOperationsResponse {
  status: ResponseStatus;
  serverOperationRecords: ServerOperationInfo[];
}

// =====================================================
// 8. ROUTING TYPES
// =====================================================

/**
 * Route Strategy Enum
 */
export const RouteStrategy = {
  WEIGHTED_ROUND_ROBIN: 'weighted-round-robin',
  CLOSE_BY_VISIT: 'close-by-visit',
} as const;

export type RouteStrategy = typeof RouteStrategy[keyof typeof RouteStrategy];

/**
 * Route Rule Status Enum
 */
export const RouteRuleStatus = {
  ACTIVE: 'active',
  INACTIVE: 'inactive',
} as const;

export type RouteRuleStatus = typeof RouteRuleStatus[keyof typeof RouteRuleStatus];

/**
 * Route Rule
 * Defines routing rules for a service
 */
export interface RouteRule {
  routeRuleId?: number;
  routeId: string;
  serviceId: string;
  name: string;
  description?: string;
  status: RouteRuleStatus;
  strategy: RouteStrategy;
  groups: ServiceGroup[];
}

/**
 * Route Rule Group
 * Associates a group with a route rule and weight
 */
export interface RouteRuleGroup {
  routeRuleId: string;
  groupId: string;
  weight: number;
  unreleasable: boolean;
  regionId?: string;
  zoneId?: string;
}

/**
 * Create Group Request
 */
export interface CreateGroupRequest {
  serviceId: string;
  regionId: string;
  zoneId: string;
  name: string;
  groupType: GroupType;
  description?: string;
}

/**
 * Create Route Rule Request
 */
export interface CreateRuleRequest {
  routeId: string;
  serviceId: string;
  name: string;
  description?: string;
  strategy: RouteStrategy;
}

/**
 * Add Rule Group Request
 */
export interface AddRuleGroupRequest {
  groupId: string;
  weight: number;
  regionId?: string;
  zoneId?: string;
}

/**
 * Update Group Request
 */
export interface UpdateGroupRequest {
  description?: string;
  status?: GroupStatus;
}

/**
 * Update Route Rule Request
 */
export interface UpdateRuleRequest {
  name?: string;
  description?: string;
  strategy?: RouteStrategy;
}

/**
 * Update Rule Group Request
 */
export interface UpdateRuleGroupRequest {
  weight: number;
}

/**
 * Add Group Tags Request
 */
export interface AddGroupTagsRequest {
  tags: GroupTag[];
}

// =====================================================
// 9. CLUSTER TYPES
// =====================================================

/**
 * Cluster Node
 */
export interface ServiceNode {
  nodeId: string;
  url: string;
  regionId: string;
  zoneId: string;
}

/**
 * Service Node Status
 */
export interface ServiceNodeStatus {
  node: ServiceNode;
  status: string; // 'starting' | 'up' | 'down' | 'unknown'
  can_service_discovery: boolean;
  can_service_registry: boolean;
  allow_registry_from_other_zone: boolean;
  allow_discovery_from_other_zone: boolean;
}

/**
 * Get Cluster Node Status Request
 */
export interface GetClusterNodeStatusRequest {
}

/**
 * Get Cluster Node Status Response
 */
export interface GetClusterNodeStatusResponse {
  node_status?: ServiceNodeStatus;
  responseStatus: ResponseStatus;
}

/**
 * Get Cluster Status Request
 */
export interface GetClusterStatusRequest {
}

/**
 * Get Cluster Status Response
 */
export interface GetClusterStatusResponse {
  nodes_status: ServiceNodeStatus[];
  node_count: number;
  responseStatus: ResponseStatus;
}

/**
 * Node Status Constants
 */
export const NodeStatusConstants = {
  STARTING: 'starting',
  UP: 'up',
  DOWN: 'down',
  UNKNOWN: 'unknown',
} as const;

// =====================================================
// 10. LEASE/STATUS TYPES
// =====================================================

/**
 * Lease Status
 */
export interface LeaseStatus {
  instance: string;
  creationTime: string;
  renewalTime: string;
  evitionTime: string;
  ttl: number;
}

/**
 * Get Leases Status Request
 */
export interface GetLeasesStatusRequest {
  serviceIds?: string[];
}

/**
 * Get Leases Status Response
 */
export interface GetLeasesStatusResponse {
  leaseUpdateMaxCount: number;
  leaseUpdateMaxCountLastUpdateTime: number;
  leaseUpdateCountLastTimeWindow: number;
  isSafe: boolean;
  isSafeCheckEnabled: boolean;
  leaseCount: number;
  leasesStatus: Record<string, LeaseStatus[]>;
  responseStatus: ResponseStatus;
}

/**
 * Get Config Status Request
 */
export interface GetConfigStatusRequest {
}

/**
 * Get Config Status Response
 */
export interface GetConfigStatusResponse {
  sources: Record<string, number>;
  properties: Record<string, string>;
  responseStatus: ResponseStatus;
}

/**
 * Get Deployment Status Request
 */
export interface GetDeploymentStatusRequest {
}

/**
 * Get Deployment Status Response
 */
export interface GetDeploymentStatusResponse {
  regionId: string;
  zoneId: string;
  appId: string;
  machineName: string;
  ip: string;
  port: number;
  protocol: string;
  path: string;
  sources: Record<string, number>;
  properties: Record<string, string>;
  responseStatus: ResponseStatus;
}

// =====================================================
// 11. AUDIT LOG TYPES
// =====================================================

/**
 * Audit Log
 * Records all management operations
 */
export interface AuditLog {
  id?: number;
  operationType: string;
  resourceType: string;
  resourceId: string;
  operatorId: string;
  operationDetails?: Record<string, any>;
  timestamp: number;
  status: 'success' | 'failure';
  errorMessage?: string;
}

/**
 * Query Logs Parameters
 */
export interface QueryLogsParams {
  operationType?: string;
  operatorId?: string;
  limit?: number;
}

/**
 * Query Instance Logs Parameters
 */
export interface QueryInstanceLogsParams {
  serviceId?: string;
  operatorId?: string;
  limit?: number;
}

/**
 * Query Server Logs Parameters
 */
export interface QueryServerLogsParams {
  serverId?: string;
  operatorId?: string;
  limit?: number;
}

/**
 * Query Group Logs Parameters
 */
export interface QueryGroupLogsParams {
  groupId?: string;
  operatorId?: string;
  limit?: number;
}

/**
 * Query Route Rule Logs Parameters
 */
export interface QueryRouteRuleLogsParams {
  rule_id?: string;
  operatorId?: string;
  limit?: number;
}

/**
 * Query Route Rule Group Logs Parameters
 */
export interface QueryRouteRuleGroupLogsParams {
  rule_id?: string;
  groupId?: string;
  operatorId?: string;
  limit?: number;
}

// =====================================================
// 12. ZONE OPERATION TYPES
// =====================================================

/**
 * Zone Operation Enum
 */
export const ZoneOperationType = {
  PULL_IN: 'pullin',
  PULL_OUT: 'pullout',
} as const;

export type ZoneOperationType = typeof ZoneOperationType[keyof typeof ZoneOperationType];

/**
 * Zone Operation Record
 */
export interface ZoneOperationRecord {
  zoneId: string;
  regionId: string;
  operation: ZoneOperationType;
  operatorId: string;
  operationTime: number;
}

/**
 * Operate Zone Request
 */
export interface OperateZoneRequest {
  zoneId: string;
  regionId: string;
  operation: ZoneOperationType;
  operatorId: string;
}

/**
 * Operate Zone Response
 */
export interface OperateZoneResponse {
  success: boolean;
  message?: string;
}

/**
 * Get Zone Status Request
 */
export interface GetZoneStatusRequest {
  zoneId: string;
  regionId: string;
}

/**
 * Get Zone Status Response
 */
export interface GetZoneStatusResponse {
  success: boolean;
  zoneId: string;
  regionId: string;
  is_down: boolean;
  operation?: ZoneOperationType;
  operatorId?: string;
}

/**
 * List Zone Operations Request
 */
export interface ListZoneOperationsRequest {
  regionId?: string;
}

/**
 * List Zone Operations Response
 */
export interface ListZoneOperationsResponse {
  success: boolean;
  operations: ZoneOperationRecord[];
}

// =====================================================
// 13. CANARY RELEASE TYPES
// =====================================================

/**
 * Canary Configuration
 */
export interface CanaryConfig {
  serviceId: string;
  ipWhitelist: string[];
  enabled: boolean;
}

/**
 * Set Canary Config Request
 */
export interface SetCanaryConfigRequest {
  serviceId: string;
  ipWhitelist: string[];
}

/**
 * Set Canary Config Response
 */
export interface SetCanaryConfigResponse {
  success: boolean;
  message?: string;
}

/**
 * Get Canary Config Request
 */
export interface GetCanaryConfigRequest {
  serviceId: string;
}

/**
 * Get Canary Config Response
 */
export interface GetCanaryConfigResponse {
  success: boolean;
  config?: CanaryConfig;
}

/**
 * Enable Canary Request
 */
export interface EnableCanaryRequest {
  serviceId: string;
  enabled: boolean;
}

/**
 * Enable Canary Response
 */
export interface EnableCanaryResponse {
  success: boolean;
  message?: string;
}

// =====================================================
// 14. REPLICATION TYPES
// =====================================================

/**
 * Replicate Register Request
 */
export interface ReplicateRegisterRequest {
  instances: Instance[];
}

/**
 * Replicate Register Response
 */
export interface ReplicateRegisterResponse {
  responseStatus: ResponseStatus;
  failed_instances?: Instance[];
}

/**
 * Replicate Heartbeat Request
 */
export interface ReplicateHeartbeatRequest {
  instance_keys: InstanceKey[];
}

/**
 * Replicate Heartbeat Response
 */
export interface ReplicateHeartbeatResponse {
  responseStatus: ResponseStatus;
  failed_instance_keys?: InstanceKey[];
}

/**
 * Replicate Unregister Request
 */
export interface ReplicateUnregisterRequest {
  instance_keys: InstanceKey[];
}

/**
 * Replicate Unregister Response
 */
export interface ReplicateUnregisterResponse {
  responseStatus: ResponseStatus;
}

/**
 * Get All Services Response
 */
export interface GetAllServicesResponse {
  responseStatus: ResponseStatus;
  services: Service[];
}

/**
 * Batch Register Request
 */
export interface BatchRegisterRequest {
  instances: Instance[];
}

/**
 * Batch Register Response
 */
export interface BatchRegisterResponse {
  responseStatus: ResponseStatus;
  failed_instances?: Instance[];
}

/**
 * Batch Heartbeat Request
 */
export interface BatchHeartbeatRequest {
  instance_keys: InstanceKey[];
}

/**
 * Batch Heartbeat Response
 */
export interface BatchHeartbeatResponse {
  responseStatus: ResponseStatus;
  failed_instance_keys?: InstanceKey[];
}

/**
 * Batch Unregister Request
 */
export interface BatchUnregisterRequest {
  instance_keys: InstanceKey[];
}

/**
 * Batch Unregister Response
 */
export interface BatchUnregisterResponse {
  responseStatus: ResponseStatus;
  failed_instance_keys?: InstanceKey[];
}

/**
 * Services Delta Request
 */
export interface ServicesDeltaRequest {
  regionId: string;
  zoneId: string;
  since_timestamp: number;
}

/**
 * Services Delta Response
 */
export interface ServicesDeltaResponse {
  responseStatus: ResponseStatus;
  services: Service[];
  current_timestamp: number;
}

/**
 * Sync Full Data Request
 */
export interface SyncFullDataRequest {
  regionId: string;
  zoneId?: string;
}

/**
 * Sync Full Data Response
 */
export interface SyncFullDataResponse {
  responseStatus: ResponseStatus;
  services: Service[];
  sync_timestamp: number;
}

// =====================================================
// 15. AUTHENTICATION TYPES
// =====================================================

/**
 * User
 */
export interface User {
  userId: string;
  username: string;
  email?: string;
  roles: Role[];
  permissions: Permission[];
  created_at?: number;
  updated_at?: number;
}

/**
 * Role
 */
export interface Role {
  roleId: string;
  name: string;
  description?: string;
  permissions: Permission[];
}

/**
 * Permission
 */
export interface Permission {
  permissionId: string;
  name: string;
  resource: string;
  action: string; // 'read' | 'write' | 'delete' | 'admin'
  description?: string;
}

/**
 * Login Request
 */
export interface LoginRequest {
  username: string;
  password: string;
}

/**
 * Login Response
 */
export interface LoginResponse {
  success: boolean;
  token?: string;
  user?: User;
  message?: string;
}

/**
 * Logout Request
 */
export interface LogoutRequest {
  token: string;
}

/**
 * Logout Response
 */
export interface LogoutResponse {
  success: boolean;
  message?: string;
}

/**
 * Refresh Token Request
 */
export interface RefreshTokenRequest {
  token: string;
}

/**
 * Refresh Token Response
 */
export interface RefreshTokenResponse {
  success: boolean;
  token?: string;
  message?: string;
}

/**
 * User Info Request
 */
export interface GetUserInfoRequest {
}

/**
 * User Info Response
 */
export interface GetUserInfoResponse {
  success: boolean;
  user?: User;
  message?: string;
}

// =====================================================
// 16. QUERY PARAMETERS & FILTERS
// =====================================================

/**
 * List Groups Query Parameters
 */
export interface ListGroupsQuery {
  serviceId?: string;
  regionId?: string;
}

/**
 * Get Group Instances Query Parameters
 */
export interface GetGroupInstancesQuery {
  regionId?: string;
  zoneId?: string;
}

// =====================================================
// 17. GROUP OPERATION TYPES
// =====================================================

/**
 * Group Operation Record
 */
export interface GroupOperation {
  operation_id?: number;
  groupId: number;
  operationType: string;
  operatorId: string;
  description?: string;
  timestamp: number;
}

/**
 * Group Operation Type Constants
 */
export const GroupOperationTypes = {
  CREATE: 'create',
  UPDATE: 'update',
  DELETE: 'delete',
  ACTIVATE: 'activate',
  DEACTIVATE: 'deactivate',
} as const;
