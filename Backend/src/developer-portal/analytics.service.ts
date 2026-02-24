import { Injectable } from '@nestjs/common';

export interface ApiUsageMetrics {
  endpoint: string;
  method: string;
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  averageResponseTime: number;
  lastAccessed: Date;
  dailyUsage: DailyUsage[];
}

export interface DailyUsage {
  date: string;
  requests: number;
  errors: number;
  averageResponseTime: number;
  uniqueUsers: number;
}

export interface DeveloperAnalytics {
  developerId: string;
  period: 'hour' | 'day' | 'week' | 'month';
  totalRequests: number;
  totalErrors: number;
  errorRate: number;
  averageResponseTime: number;
  topEndpoints: ApiUsageMetrics[];
  userGrowth: UserGrowthMetrics;
  costAnalysis: CostAnalysis;
}

export interface UserGrowthMetrics {
  newDevelopers: number;
  activeDevelopers: number;
  retainedDevelopers: number;
  churnRate: number;
}

export interface CostAnalysis {
  totalApiCalls: number;
  estimatedCost: number;
  costPerCall: number;
  costBreakdown: CostBreakdown[];
}

export interface CostBreakdown {
  category: string;
  cost: number;
  percentage: number;
}

export interface ApiUsageAlert {
  id: string;
  developerId: string;
  type: 'rate_limit' | 'error_spike' | 'usage_anomaly' | 'cost_threshold';
  severity: 'low' | 'medium' | 'high' | 'critical';
  message: string;
  threshold: number;
  currentValue: number;
  timestamp: Date;
  resolved: boolean;
}

@Injectable()
export class AnalyticsService {
  private usageData: Map<string, ApiUsageMetrics[]> = new Map();
  private alerts: Map<string, ApiUsageAlert[]> = new Map();

  async trackApiUsage(
    developerId: string,
    endpoint: string,
    method: string,
    responseTime: number,
    success: boolean
  ): Promise<void> {
    const metrics = this.getOrCreateMetrics(developerId, endpoint, method);
    
    metrics.totalRequests++;
    metrics.lastAccessed = new Date();
    
    if (success) {
      metrics.successfulRequests++;
    } else {
      metrics.failedRequests++;
    }

    // Update average response time
    metrics.averageResponseTime = this.calculateAverageResponseTime(
      metrics.averageResponseTime,
      responseTime,
      metrics.totalRequests
    );

    // Update daily usage
    await this.updateDailyUsage(metrics, responseTime, success);

    // Check for alerts
    await this.checkForAlerts(developerId, metrics);

    // Save to database (mock implementation)
    this.saveMetrics(developerId, metrics);
  }

  async getDeveloperAnalytics(
    developerId: string,
    period: 'hour' | 'day' | 'week' | 'month' = 'day'
  ): Promise<DeveloperAnalytics> {
    const metrics = Array.from(this.usageData.get(developerId)?.values() || []);
    
    const totalRequests = metrics.reduce((sum, m) => sum + m.totalRequests, 0);
    const totalErrors = metrics.reduce((sum, m) => sum + m.failedRequests, 0);
    const errorRate = totalRequests > 0 ? (totalErrors / totalRequests) * 100 : 0;
    const averageResponseTime = this.calculateOverallAverageResponseTime(metrics);

    const topEndpoints = metrics
      .sort((a, b) => b.totalRequests - a.totalRequests)
      .slice(0, 10);

    const userGrowth = await this.calculateUserGrowth(developerId, period);
    const costAnalysis = await this.calculateCostAnalysis(developerId, period);

    return {
      developerId,
      period,
      totalRequests,
      totalErrors,
      errorRate,
      averageResponseTime,
      topEndpoints,
      userGrowth,
      costAnalysis,
    };
  }

  async getUsageTrends(
    developerId: string,
    days: number = 30
  ): Promise<DailyUsage[]> {
    const metrics = this.usageData.get(developerId);
    if (!metrics) return [];

    const allDailyUsage: DailyUsage[] = [];
    
    metrics.forEach(metric => {
      allDailyUsage.push(...metric.dailyUsage);
    });

    // Sort by date and limit to specified days
    return allDailyUsage
      .sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime())
      .slice(0, days);
  }

  async generateUsageReport(
    developerId: string,
    startDate: Date,
    endDate: Date
  ): Promise<{
    summary: DeveloperAnalytics;
    trends: DailyUsage[];
    alerts: ApiUsageAlert[];
    recommendations: string[];
  }> {
    const analytics = await this.getDeveloperAnalytics(developerId);
    const trends = await this.getUsageTrends(developerId);
    const alerts = this.getAlerts(developerId);
    const recommendations = this.generateRecommendations(analytics, trends, alerts);

    return {
      summary: analytics,
      trends,
      alerts,
      recommendations,
    };
  }

  async setUsageAlert(
    developerId: string,
    type: ApiUsageAlert['type'],
    threshold: number
  ): Promise<ApiUsageAlert> {
    const alert: ApiUsageAlert = {
      id: this.generateAlertId(),
      developerId,
      type,
      severity: 'medium',
      message: `Alert configured for ${type} with threshold ${threshold}`,
      threshold,
      currentValue: 0,
      timestamp: new Date(),
      resolved: false,
    };

    const developerAlerts = this.alerts.get(developerId) || [];
    developerAlerts.push(alert);
    this.alerts.set(developerId, developerAlerts);

    return alert;
  }

  async getAlerts(developerId: string): Promise<ApiUsageAlert[]> {
    return this.alerts.get(developerId) || [];
  }

  async resolveAlert(alertId: string): Promise<boolean> {
    for (const [developerId, alerts] of this.alerts.entries()) {
      const alert = alerts.find(a => a.id === alertId);
      if (alert) {
        alert.resolved = true;
        return true;
      }
    }
    return false;
  }

  private getOrCreateMetrics(
    developerId: string,
    endpoint: string,
    method: string
  ): ApiUsageMetrics {
    const key = `${developerId}:${endpoint}:${method}`;
    const developerMetrics = this.usageData.get(developerId) || new Map();
    
    let metrics = developerMetrics.get(key);
    
    if (!metrics) {
      metrics = {
        endpoint,
        method,
        totalRequests: 0,
        successfulRequests: 0,
        failedRequests: 0,
        averageResponseTime: 0,
        lastAccessed: new Date(),
        dailyUsage: [],
      };
      
      developerMetrics.set(key, metrics);
      this.usageData.set(developerId, developerMetrics);
    }

    return metrics;
  }

  private calculateAverageResponseTime(
    currentAverage: number,
    newResponseTime: number,
    totalRequests: number
  ): number {
    return ((currentAverage * (totalRequests - 1)) + newResponseTime) / totalRequests;
  }

  private calculateOverallAverageResponseTime(metrics: ApiUsageMetrics[]): number {
    if (metrics.length === 0) return 0;
    
    const totalResponseTime = metrics.reduce((sum, m) => sum + m.averageResponseTime, 0);
    return totalResponseTime / metrics.length;
  }

  private async updateDailyUsage(
    metrics: ApiUsageMetrics,
    responseTime: number,
    success: boolean
  ): Promise<void> {
    const today = new Date().toISOString().split('T')[0];
    let dailyUsage = metrics.dailyUsage.find(usage => usage.date === today);

    if (!dailyUsage) {
      dailyUsage = {
        date: today,
        requests: 0,
        errors: 0,
        averageResponseTime: 0,
        uniqueUsers: 0,
      };
      metrics.dailyUsage.push(dailyUsage);
    }

    dailyUsage.requests++;
    if (!success) {
      dailyUsage.errors++;
    }

    dailyUsage.averageResponseTime = this.calculateAverageResponseTime(
      dailyUsage.averageResponseTime,
      responseTime,
      dailyUsage.requests
    );
  }

  private async checkForAlerts(
    developerId: string,
    metrics: ApiUsageMetrics
  ): Promise<void> {
    const alerts = this.alerts.get(developerId) || [];

    // Check error rate alert
    const errorRate = metrics.totalRequests > 0 
      ? (metrics.failedRequests / metrics.totalRequests) * 100 
      : 0;

    const errorAlert = alerts.find(a => a.type === 'error_spike');
    if (errorAlert && errorRate > errorAlert.threshold) {
      await this.triggerAlert({
        ...errorAlert,
        currentValue: errorRate,
        message: `Error rate (${errorRate.toFixed(2)}%) exceeded threshold (${errorAlert.threshold}%)`,
        severity: errorRate > 10 ? 'critical' : 'high',
      });
    }

    // Check rate limit alert
    const rateLimitAlert = alerts.find(a => a.type === 'rate_limit');
    if (rateLimitAlert && metrics.totalRequests > rateLimitAlert.threshold) {
      await this.triggerAlert({
        ...rateLimitAlert,
        currentValue: metrics.totalRequests,
        message: `Request count (${metrics.totalRequests}) exceeded rate limit threshold (${rateLimitAlert.threshold})`,
        severity: 'medium',
      });
    }
  }

  private async triggerAlert(alert: ApiUsageAlert): Promise<void> {
    // In a real implementation, this would send notifications
    console.warn(`API Usage Alert: ${alert.message}`);
    
    // Store alert in database
    const developerAlerts = this.alerts.get(alert.developerId) || [];
    developerAlerts.push(alert);
    this.alerts.set(alert.developerId, developerAlerts);
  }

  private async calculateUserGrowth(
    developerId: string,
    period: 'hour' | 'day' | 'week' | 'month'
  ): Promise<UserGrowthMetrics> {
    // Mock implementation - in real system, query user activity data
    return {
      newDevelopers: Math.floor(Math.random() * 10),
      activeDevelopers: Math.floor(Math.random() * 100) + 50,
      retainedDevelopers: Math.floor(Math.random() * 80) + 20,
      churnRate: Math.random() * 5,
    };
  }

  private async calculateCostAnalysis(
    developerId: string,
    period: 'hour' | 'day' | 'week' | 'month'
  ): Promise<CostAnalysis> {
    const metrics = Array.from(this.usageData.get(developerId)?.values() || []);
    const totalApiCalls = metrics.reduce((sum, m) => sum + m.totalRequests, 0);
    
    // Mock pricing - in real implementation, use actual pricing tiers
    const costPerCall = 0.001; // $0.001 per API call
    const estimatedCost = totalApiCalls * costPerCall;

    const costBreakdown: CostBreakdown[] = [
      {
        category: 'API Calls',
        cost: estimatedCost,
        percentage: 100,
      },
    ];

    return {
      totalApiCalls,
      estimatedCost,
      costPerCall,
      costBreakdown,
    };
  }

  private generateRecommendations(
    analytics: DeveloperAnalytics,
    trends: DailyUsage[],
    alerts: ApiUsageAlert[]
  ): string[] {
    const recommendations: string[] = [];

    // Error rate recommendations
    if (analytics.errorRate > 5) {
      recommendations.push('Consider implementing better error handling to reduce the ' +
        `${analytics.errorRate.toFixed(2)}% error rate`);
    }

    // Response time recommendations
    if (analytics.averageResponseTime > 1000) {
      recommendations.push('Average response time is high. Consider optimizing API calls or ' +
        'implementing caching strategies');
    }

    // Usage pattern recommendations
    const recentUsage = trends.slice(-7); // Last 7 days
    const averageDailyUsage = recentUsage.reduce((sum, day) => sum + day.requests, 0) / 7;
    
    if (averageDailyUsage > 1000) {
      recommendations.push('High usage detected. Consider implementing request batching ' +
        'to reduce costs and improve efficiency');
    }

    // Alert-based recommendations
    const unresolvedAlerts = alerts.filter(alert => !alert.resolved);
    if (unresolvedAlerts.length > 0) {
      recommendations.push(`You have ${unresolvedAlerts.length} unresolved alerts. ` +
        'Please review and address them promptly.');
    }

    return recommendations;
  }

  private saveMetrics(developerId: string, metrics: ApiUsageMetrics): void {
    // In a real implementation, save to database
    // For now, just store in memory
    const developerMetrics = this.usageData.get(developerId) || new Map();
    const key = `${metrics.endpoint}:${metrics.method}`;
    developerMetrics.set(key, metrics);
    this.usageData.set(developerId, developerMetrics);
  }

  private generateAlertId(): string {
    return `alert_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}
