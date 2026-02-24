import { Injectable } from '@nestjs/common';

export interface OnboardingStep {
  id: string;
  title: string;
  description: string;
  type: 'tutorial' | 'video' | 'interactive' | 'quiz';
  content: string;
  duration: number; // in minutes
  required: boolean;
  order: number;
}

export interface DeveloperProfile {
  id: string;
  name: string;
  email: string;
  company?: string;
  apiKey: string;
  permissions: string[];
  onboardingProgress: OnboardingProgress;
  createdAt: Date;
  lastActiveAt: Date;
}

export interface OnboardingProgress {
  completedSteps: string[];
  currentStep: string;
  totalSteps: number;
  completedPercentage: number;
  estimatedTimeRemaining: number; // in minutes
  startedAt: Date;
  lastActivityAt: Date;
}

export interface OnboardingGuide {
  id: string;
  title: string;
  description: string;
  difficulty: 'beginner' | 'intermediate' | 'advanced';
  category: 'getting-started' | 'api-basics' | 'advanced-features' | 'best-practices';
  steps: OnboardingStep[];
  estimatedTotalTime: number;
  prerequisites: string[];
}

@Injectable()
export class OnboardingService {
  private onboardingGuides: Map<string, OnboardingGuide> = new Map();
  private developerProfiles: Map<string, DeveloperProfile> = new Map();

  constructor() {
    this.initializeGuides();
  }

  private initializeGuides(): void {
    const gettingStartedGuide: OnboardingGuide = {
      id: 'getting-started',
      title: 'Getting Started with Stellara API',
      description: 'Complete guide to start using the Stellara API effectively',
      difficulty: 'beginner',
      category: 'getting-started',
      estimatedTotalTime: 45,
      prerequisites: [],
      steps: [
        {
          id: 'account-setup',
          title: 'Account Setup',
          description: 'Create your developer account and generate API keys',
          type: 'tutorial',
          content: 'Step-by-step guide for account creation...',
          duration: 10,
          required: true,
          order: 1,
        },
        {
          id: 'api-key-generation',
          title: 'API Key Generation',
          description: 'Generate and secure your API keys',
          type: 'interactive',
          content: 'Interactive tutorial for API key generation...',
          duration: 15,
          required: true,
          order: 2,
        },
        {
          id: 'first-api-call',
          title: 'Making Your First API Call',
          description: 'Execute your first transaction using the API',
          type: 'interactive',
          content: 'Hands-on tutorial for first API call...',
          duration: 20,
          required: true,
          order: 3,
        },
      ],
    };

    const apiBasicsGuide: OnboardingGuide = {
      id: 'api-basics',
      title: 'API Fundamentals',
      description: 'Master the core concepts of the Stellara API',
      difficulty: 'beginner',
      category: 'api-basics',
      estimatedTotalTime: 60,
      prerequisites: ['getting-started'],
      steps: [
        {
          id: 'authentication',
          title: 'Authentication Methods',
          description: 'Learn about API key and OAuth authentication',
          type: 'tutorial',
          content: 'Detailed authentication guide...',
          duration: 15,
          required: true,
          order: 1,
        },
        {
          id: 'request-response',
          title: 'Request/Response Format',
          description: 'Understanding API request and response structures',
          type: 'tutorial',
          content: 'Request/response format documentation...',
          duration: 20,
          required: true,
          order: 2,
        },
        {
          id: 'error-handling',
          title: 'Error Handling',
          description: 'Proper error handling and status codes',
          type: 'tutorial',
          content: 'Error handling best practices...',
          duration: 15,
          required: true,
          order: 3,
        },
        {
          id: 'rate-limits',
          title: 'Rate Limits and Quotas',
          description: 'Understanding API rate limits and usage quotas',
          type: 'tutorial',
          content: 'Rate limits documentation...',
          duration: 10,
          required: true,
          order: 4,
        },
      ],
    };

    this.onboardingGuides.set('getting-started', gettingStartedGuide);
    this.onboardingGuides.set('api-basics', apiBasicsGuide);
  }

  async startOnboarding(developerId: string, guideId: string): Promise<OnboardingProgress> {
    const guide = this.onboardingGuides.get(guideId);
    if (!guide) {
      throw new Error(`Guide not found: ${guideId}`);
    }

    const progress: OnboardingProgress = {
      completedSteps: [],
      currentStep: guide.steps[0].id,
      totalSteps: guide.steps.length,
      completedPercentage: 0,
      estimatedTimeRemaining: guide.estimatedTotalTime,
      startedAt: new Date(),
      lastActivityAt: new Date(),
    };

    // Save progress to database (mock implementation)
    this.updateDeveloperProgress(developerId, progress);

    return progress;
  }

  async completeStep(developerId: string, stepId: string): Promise<OnboardingProgress> {
    const profile = this.developerProfiles.get(developerId);
    if (!profile) {
      throw new Error('Developer profile not found');
    }

    const progress = profile.onboardingProgress;
    
    if (!progress.completedSteps.includes(stepId)) {
      progress.completedSteps.push(stepId);
      progress.completedPercentage = (progress.completedSteps.length / progress.totalSteps) * 100;
      progress.lastActivityAt = new Date();

      // Update current step to next uncompleted step
      const currentGuide = this.getCurrentGuide(progress.currentStep);
      if (currentGuide) {
        const nextStep = currentGuide.steps.find(step => !progress.completedSteps.includes(step.id));
        if (nextStep) {
          progress.currentStep = nextStep.id;
        }
      }

      // Update estimated time remaining
      progress.estimatedTimeRemaining = this.calculateRemainingTime(progress);

      // Save updated progress
      this.updateDeveloperProgress(developerId, progress);
    }

    return progress;
  }

  async getOnboardingProgress(developerId: string): Promise<OnboardingProgress | null> {
    const profile = this.developerProfiles.get(developerId);
    return profile ? profile.onboardingProgress : null;
  }

  async getAvailableGuides(): Promise<OnboardingGuide[]> {
    return Array.from(this.onboardingGuides.values());
  }

  async getGuideById(guideId: string): Promise<OnboardingGuide | null> {
    return this.onboardingGuides.get(guideId) || null;
  }

  async getRecommendedGuides(developerId: string): Promise<OnboardingGuide[]> {
    const profile = this.developerProfiles.get(developerId);
    const progress = profile?.onboardingProgress;

    if (!progress || progress.completedPercentage === 100) {
      // Return advanced guides for completed developers
      return Array.from(this.onboardingGuides.values())
        .filter(guide => guide.difficulty === 'advanced');
    }

    // Return next logical guide based on current progress
    const currentGuide = this.getCurrentGuide(progress.currentStep);
    if (!currentGuide) {
      return [];
    }

    const completedGuides = this.getCompletedGuides(developerId);
    
    return Array.from(this.onboardingGuides.values())
      .filter(guide => 
        guide.prerequisites.every(prereq => completedGuides.includes(prereq))
      );
  }

  async generatePersonalizedPath(developerId: string): Promise<OnboardingGuide[]> {
    const profile = this.developerProfiles.get(developerId);
    
    if (!profile) {
      // Return beginner path for new developers
      return [this.onboardingGuides.get('getting-started')!];
    }

    const progress = profile.onboardingProgress;
    const completedGuides = this.getCompletedGuides(developerId);

    // Analyze developer's skill level and interests
    const skillLevel = this.assessSkillLevel(progress);
    const interests = this.analyzeInterests(profile);

    // Generate personalized learning path
    return this.generateLearningPath(skillLevel, interests, completedGuides);
  }

  private getCurrentGuide(currentStepId: string): OnboardingGuide | null {
    for (const guide of this.onboardingGuides.values()) {
      if (guide.steps.some(step => step.id === currentStepId)) {
        return guide;
      }
    }
    return null;
  }

  private getCompletedGuides(developerId: string): string[] {
    const profile = this.developerProfiles.get(developerId);
    if (!profile) return [];

    const completed: string[] = [];
    
    for (const guide of this.onboardingGuides.values()) {
      const allStepsCompleted = guide.steps.every(step => 
        profile.onboardingProgress.completedSteps.includes(step.id)
      );
      
      if (allStepsCompleted) {
        completed.push(guide.id);
      }
    }

    return completed;
  }

  private calculateRemainingTime(progress: OnboardingProgress): number {
    const completedSteps = progress.completedSteps.length;
    const totalSteps = progress.totalSteps;
    const averageStepTime = 15; // Average time per step in minutes
    
    return (totalSteps - completedSteps) * averageStepTime;
  }

  private assessSkillLevel(progress: OnboardingProgress): 'beginner' | 'intermediate' | 'advanced' {
    const percentage = progress.completedPercentage;
    
    if (percentage < 30) return 'beginner';
    if (percentage < 70) return 'intermediate';
    return 'advanced';
  }

  private analyzeInterests(profile: DeveloperProfile): string[] {
    // In a real implementation, this would analyze API usage patterns
    // For now, return default interests based on permissions
    const interests: string[] = [];
    
    if (profile.permissions.includes('transactions')) {
      interests.push('blockchain', 'payments');
    }
    
    if (profile.permissions.includes('smart-contracts')) {
      interests.push('defi', 'contracts');
    }
    
    return interests;
  }

  private generateLearningPath(
    skillLevel: string,
    interests: string[],
    completedGuides: string[]
  ): OnboardingGuide[] {
    const allGuides = Array.from(this.onboardingGuides.values());
    
    return allGuides
      .filter(guide => 
        !completedGuides.includes(guide.id) &&
        guide.prerequisites.every(prereq => completedGuides.includes(prereq))
      )
      .sort((a, b) => {
        // Prioritize guides matching interests
        const aInterestMatch = this.calculateInterestMatch(a, interests);
        const bInterestMatch = this.calculateInterestMatch(b, interests);
        
        if (aInterestMatch !== bInterestMatch) {
          return bInterestMatch - aInterestMatch;
        }
        
        // Then by difficulty
        const difficultyOrder = { beginner: 1, intermediate: 2, advanced: 3 };
        return difficultyOrder[a.difficulty] - difficultyOrder[b.difficulty];
      });
  }

  private calculateInterestMatch(guide: OnboardingGuide, interests: string[]): number {
    const categoryKeywords = {
      'getting-started': ['setup', 'account', 'basics'],
      'api-basics': ['api', 'requests', 'authentication'],
      'advanced-features': ['advanced', 'optimization', 'performance'],
      'best-practices': ['security', 'patterns', 'architecture'],
    };

    const guideKeywords = categoryKeywords[guide.category] || [];
    return guideKeywords.filter(keyword => 
      interests.some(interest => 
        interest.toLowerCase().includes(keyword.toLowerCase())
      )
    ).length;
  }

  private updateDeveloperProgress(developerId: string, progress: OnboardingProgress): void {
    const profile = this.developerProfiles.get(developerId);
    if (profile) {
      profile.onboardingProgress = progress;
      profile.lastActiveAt = new Date();
    }
  }

  async createDeveloperProfile(profileData: Omit<DeveloperProfile, 'id' | 'onboardingProgress' | 'createdAt' | 'lastActiveAt'>): Promise<DeveloperProfile> {
    const profile: DeveloperProfile = {
      id: this.generateId(),
      ...profileData,
      onboardingProgress: {
        completedSteps: [],
        currentStep: '',
        totalSteps: 0,
        completedPercentage: 0,
        estimatedTimeRemaining: 0,
        startedAt: new Date(),
        lastActivityAt: new Date(),
      },
      createdAt: new Date(),
      lastActiveAt: new Date(),
    };

    this.developerProfiles.set(profile.id, profile);
    return profile;
  }

  private generateId(): string {
    return `dev_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}
