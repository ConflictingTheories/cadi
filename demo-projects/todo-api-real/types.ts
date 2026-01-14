// Type definitions for Todo system
export interface Todo {
    id: string;
    title: string;
    description?: string;
    completed: boolean;
    dueDate?: string;
    priority: 'low' | 'medium' | 'high';
    tags?: string[];
    created_at: string;
    updated_at: string;
}

export interface CreateTodoInput {
    title: string;
    description?: string;
    priority?: 'low' | 'medium' | 'high';
    dueDate?: string;
    tags?: string[];
}

export interface UpdateTodoInput {
    title?: string;
    completed?: boolean;
    priority?: 'low' | 'medium' | 'high';
    description?: string;
    dueDate?: string;
    tags?: string[];
}

export interface ApiResponse<T> {
    success: boolean;
    data?: T;
    error?: string;
    count?: number;
}
