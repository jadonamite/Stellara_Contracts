import { Injectable } from '@nestjs/common';

export interface Region {
  id: string;
  name: string;
  code: string;
  endpoint: string;
  databaseUrl: string;
  isActive: boolean;
  priority: number; // 1 = highest priority
  location: {
    country: string;
    city: string;
    latitude: number;
    longitude: number;
  };
  healthCheck: {
    url: string;
    interval: number;
    timeout: number;
    retries: number;
  };
  capabilities: {
    supportsTransactions: boolean;
    supportsSmartContracts: boolean;
    supportsBatching: boolean;
    maxThroughput: number;
  };
}

export interface DeploymentStatus {
  totalRegions: number;
  activeRegions: number;
  healthyRegions: number;
  lastFailover: Date | null;
  uptime: number;
  averageResponseTime: number;
}

export interface FailoverConfig {
  enabled: boolean;
  healthCheckInterval: number;
  failoverThreshold: number; // consecutive failures before failover
  automaticFailback: boolean;
  failbackDelay: number; // milliseconds
  trafficDistribution: 'round-robin' | 'weighted' | 'geographic' | 'health-based';
}

export interface DisasterRecoveryPlan {
  id: string;
  name: string;
  description: string;
  regions: string[];
  rpo: number; // Recovery Point Objective in minutes
  rto: number; // Recovery Time Objective in minutes
  backupFrequency: number; // in hours
  testFrequency: number; // in days
  lastTested: Date;
  isActive: boolean;
}

@Injectable()
export class MultiRegionService {
  private regions: Map<string, Region> = new Map();
  private failoverConfig: FailoverConfig;
  private currentPrimaryRegion: string;
  private deploymentStatus: DeploymentStatus;

  constructor() {
    this.initializeRegions();
    this.initializeFailoverConfig();
    this.startHealthMonitoring();
  }

  private initializeRegions(): void {
    const regions: Region[] = [
      {
        id: 'us-east-1',
        name: 'US East (N. Virginia)',
        code: 'USE1',
        endpoint: 'https://api.stellara.io',
        databaseUrl: 'postgresql://us-east-db.stellara.io',
        isActive: true,
        priority: 1,
        location: {
          country: 'United States',
          city: 'Ashburn',
          latitude: 39.0437,
          longitude: -77.4875,
        },
        healthCheck: {
          url: 'https://api.stellara.io/health',
          interval: 30000, // 30 seconds
          timeout: 5000, // 5 seconds
          retries: 3,
        },
        capabilities: {
          supportsTransactions: true,
          supportsSmartContracts: true,
          supportsBatching: true,
          maxThroughput: 10000, // requests per second
        },
      },
      {
        id: 'eu-west-1',
        name: 'EU West (Ireland)',
        code: 'EUW1',
        endpoint: 'https://api-eu.stellara.io',
        databaseUrl: 'postgresql://eu-west-db.stellara.io',
        isActive: true,
        priority: 2,
        location: {
          country: 'Ireland',
          city: 'Dublin',
          latitude: 53.3498,
          longitude: -6.2603,
        },
        healthCheck: {
          url: 'https://api-eu.stellara.io/health',
          interval: 30000,
          timeout: 5000,
          retries: 3,
        },
        capabilities: {
          supportsTransactions: true,
          supportsSmartContracts: true,
          supportsBatching: true,
          maxThroughput: 8000,
        },
      },
      {
        id: 'ap-southeast-1',
        name: 'AP Southeast (Singapore)',
        code: 'APSE1',
        endpoint: 'https://api-ap.stellara.io',
        databaseUrl: 'postgresql://ap-southeast-db.stellara.io',
        isActive: true,
        priority: 3,
        location: {
          country: 'Singapore',
          city: 'Singapore',
          latitude: 1.3521,
          longitude: 103.8198,
        },
        healthCheck: {
          url: 'https://api-ap.stellara.io/health',
          interval: 30000,
          timeout: 5000,
          retries: 3,
        },
        capabilities: {
          supportsTransactions: true,
          supportsSmartContracts: false,
          supportsBatching: true,
          maxThroughput: 6000,
        },
      },
    ];

    regions.forEach(region => {
      this.regions.set(region.id, region);
    });

    this.currentPrimaryRegion = 'us-east-1'; // Default primary
  }

  private initializeFailoverConfig(): void {
    this.failoverConfig = {
      enabled: true,
      healthCheckInterval: 30000, // 30 seconds
      failoverThreshold: 3, // 3 consecutive failures
      automaticFailback: true,
      failbackDelay: 300000, // 5 minutes
      trafficDistribution: 'health-based',
    };
  }

  async deployToRegion(regionId: string): Promise<Region> {
    const region = this.regions.get(regionId);
    if (!region) {
      throw new Error(`Region not found: ${regionId}`);
    }

    // Simulate deployment process
    console.log(`Deploying to region: ${region.name}`);
    
    // In a real implementation, this would:
    // 1. Provision infrastructure
    // 2. Deploy application code
    // 3. Configure database
    // 4. Setup monitoring
    // 5. Run health checks

    region.isActive = true;
    this.regions.set(regionId, region);

    return region;
  }

  async enableRegion(regionId: string): Promise<boolean> {
    const region = this.regions.get(regionId);
    if (!region) return false;

    region.isActive = true;
    console.log(`Region enabled: ${region.name}`);
    return true;
  }

  async disableRegion(regionId: string): Promise<boolean> {
    const region = this.regions.get(regionId);
    if (!region) return false;

    region.isActive = false;
    console.log(`Region disabled: ${region.name}`);
    
    // If disabling primary region, trigger failover
    if (regionId === this.currentPrimaryRegion) {
      await this.performFailover();
    }

    return true;
  }

  async getOptimalRegion(userLocation?: { latitude: number; longitude: number }): Promise<Region> {
    if (!userLocation) {
      // Return highest priority healthy region
      return this.getHighestPriorityHealthyRegion();
    }

    // Find closest region to user
    let closestRegion: Region | null = null;
    let minDistance = Infinity;

    for (const region of this.regions.values()) {
      if (!region.isActive) continue;

      const distance = this.calculateDistance(
        userLocation,
        region.location
      );

      if (distance < minDistance) {
        minDistance = distance;
        closestRegion = region;
      }
    }

    return closestRegion || this.getHighestPriorityHealthyRegion();
  }

  async getDeploymentStatus(): Promise<DeploymentStatus> {
    const regions = Array.from(this.regions.values());
    const activeRegions = regions.filter(r => r.isActive);
    const healthyRegions = await this.getHealthyRegions();

    this.deploymentStatus = {
      totalRegions: regions.length,
      activeRegions: activeRegions.length,
      healthyRegions: healthyRegions.length,
      lastFailover: null, // Would be tracked in real implementation
      uptime: 99.9, // Mock uptime percentage
      averageResponseTime: 150, // Mock response time in ms
    };

    return this.deploymentStatus;
  }

  async updateRegionConfig(regionId: string, config: Partial<Region>): Promise<Region> {
    const region = this.regions.get(regionId);
    if (!region) {
      throw new Error(`Region not found: ${regionId}`);
    }

    const updatedRegion = { ...region, ...config };
    this.regions.set(regionId, updatedRegion);

    return updatedRegion;
  }

  async testRegionConnectivity(regionId: string): Promise<{
    connected: boolean;
    responseTime: number;
    error?: string;
  }> {
    const region = this.regions.get(regionId);
    if (!region) {
      return {
        connected: false,
        responseTime: 0,
        error: 'Region not found',
      };
    }

    const startTime = Date.now();

    try {
      // Test connectivity to region's health endpoint
      const response = await fetch(region.healthCheck.url, {
        method: 'GET',
        timeout: region.healthCheck.timeout,
      });

      const responseTime = Date.now() - startTime;

      if (response.ok) {
        return {
          connected: true,
          responseTime,
        };
      } else {
        return {
          connected: false,
          responseTime,
          error: `HTTP ${response.status}`,
        };
      }
    } catch (error) {
      return {
        connected: false,
        responseTime: Date.now() - startTime,
        error: error.message,
      };
    }
  }

  async performFailover(): Promise<string> {
    console.log('Performing automatic failover...');

    const newPrimaryRegion = this.getHighestPriorityHealthyRegion();
    if (!newPrimaryRegion) {
      throw new Error('No healthy regions available for failover');
    }

    const oldPrimaryRegion = this.currentPrimaryRegion;
    this.currentPrimaryRegion = newPrimaryRegion.id;

    console.log(`Failover completed: ${oldPrimaryRegion} -> ${newPrimaryRegion.id}`);

    // Update traffic routing
    await this.updateTrafficRouting(newPrimaryRegion.id);

    return newPrimaryRegion.id;
  }

  async performFailback(): Promise<string | null> {
    if (!this.failoverConfig.automaticFailback) {
      return null;
    }

    const originalRegion = this.regions.get('us-east-1'); // Original primary
    if (!originalRegion || !originalRegion.isActive) {
      return null;
    }

    // Wait for configured delay
    await new Promise(resolve => 
      setTimeout(resolve, this.failoverConfig.failbackDelay)
    );

    // Check if original region is healthy
    const connectivity = await this.testRegionConnectivity(originalRegion.id);
    if (!connectivity.connected) {
      return null;
    }

    this.currentPrimaryRegion = originalRegion.id;
    console.log(`Failback completed: ${this.currentPrimaryRegion} -> ${originalRegion.id}`);

    await this.updateTrafficRouting(originalRegion.id);
    return originalRegion.id;
  }

  private async getHealthyRegions(): Promise<Region[]> {
    const healthyRegions: Region[] = [];

    for (const region of this.regions.values()) {
      if (!region.isActive) continue;

      const connectivity = await this.testRegionConnectivity(region.id);
      if (connectivity.connected) {
        healthyRegions.push(region);
      }
    }

    return healthyRegions;
  }

  private getHighestPriorityHealthyRegion(): Region | null {
    const healthyRegions = Array.from(this.regions.values())
      .filter(region => region.isActive);

    if (healthyRegions.length === 0) return null;

    return healthyRegions
      .sort((a, b) => a.priority - b.priority)[0];
  }

  private calculateDistance(
    point1: { latitude: number; longitude: number },
    point2: { latitude: number; longitude: number }
  ): number {
    const R = 6371; // Earth's radius in km
    const dLat = this.toRadians(point2.latitude - point1.latitude);
    const dLon = this.toRadians(point2.longitude - point1.longitude);

    const a = 
      Math.sin(dLat / 2) * Math.sin(dLat / 2) +
      Math.cos(this.toRadians(point1.latitude)) * 
      Math.cos(this.toRadians(point2.latitude)) * 
      Math.sin(dLon / 2) * Math.sin(dLon / 2);

    const c = 2 * Math.atan2(
      Math.sqrt(a),
      Math.sqrt(1 - a)
    );

    return R * c;
  }

  private toRadians(degrees: number): number {
    return degrees * (Math.PI / 180);
  }

  private async updateTrafficRouting(primaryRegionId: string): Promise<void> {
    // In a real implementation, this would update load balancer
    // DNS records, or API gateway configuration
    console.log(`Traffic routing updated to primary region: ${primaryRegionId}`);
  }

  private startHealthMonitoring(): void {
    setInterval(async () => {
      await this.performHealthChecks();
    }, this.failoverConfig.healthCheckInterval);
  }

  private async performHealthChecks(): Promise<void> {
    const primaryRegion = this.regions.get(this.currentPrimaryRegion);
    if (!primaryRegion) return;

    let consecutiveFailures = 0;

    for (let i = 0; i < primaryRegion.healthCheck.retries; i++) {
      const connectivity = await this.testRegionConnectivity(primaryRegion.id);
      
      if (!connectivity.connected) {
        consecutiveFailures++;
      } else {
        consecutiveFailures = 0;
        break;
      }
    }

    // Trigger failover if threshold exceeded
    if (consecutiveFailures >= this.failoverConfig.failoverThreshold) {
      await this.performFailover();
    }
  }

  async getAllRegions(): Promise<Region[]> {
    return Array.from(this.regions.values());
  }

  async getRegionById(regionId: string): Promise<Region | null> {
    return this.regions.get(regionId) || null;
  }
}
