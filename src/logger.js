// Frontend logging wrapper that sends logs to the backend
import { invoke } from '@tauri-apps/api/core';

class Logger {
    constructor() {
        this.prefix = '[FRONTEND]';
    }

    async debug(message, ...args) {
        const fullMessage = this.formatMessage(message, args);
        console.debug(this.prefix, fullMessage);

        try {
            await invoke('log_frontend_debug', { message: fullMessage });
        } catch (error) {
            console.error('Failed to send debug log to backend:', error);
        }
    }

    async info(message, ...args) {
        const fullMessage = this.formatMessage(message, args);
        console.info(this.prefix, fullMessage);

        try {
            await invoke('log_frontend_info', { message: fullMessage });
        } catch (error) {
            console.error('Failed to send info log to backend:', error);
        }
    }

    async warn(message, ...args) {
        const fullMessage = this.formatMessage(message, args);
        console.warn(this.prefix, fullMessage);

        try {
            await invoke('log_frontend_warn', { message: fullMessage });
        } catch (error) {
            console.error('Failed to send warn log to backend:', error);
        }
    }

    async error(message, ...args) {
        const fullMessage = this.formatMessage(message, args);
        console.error(this.prefix, fullMessage);

        try {
            await invoke('log_frontend_error', { message: fullMessage });
        } catch (error) {
            console.error('Failed to send error log to backend:', error);
        }
    }

    formatMessage(message, args) {
        if (args.length === 0) {
            return message;
        }

        // Handle objects and arrays by JSON stringifying them
        const formattedArgs = args.map(arg => {
            if (typeof arg === 'object' && arg !== null) {
                try {
                    return JSON.stringify(arg);
                } catch (e) {
                    return String(arg);
                }
            }
            return String(arg);
        });

        return `${message} ${formattedArgs.join(' ')}`;
    }

    async getLogFiles() {
        try {
            return await invoke('get_log_files_list');
        } catch (error) {
            console.error('Failed to get log files:', error);
            return [];
        }
    }

    // Convenience methods for common logging patterns
    async logServerResponse(action, response) {
        await this.debug(`Server response for ${action}:`, response);
    }

    async logUserAction(action, details = {}) {
        await this.info(`User action: ${action}`, details);
    }

    async logApiCall(endpoint, params = {}) {
        await this.debug(`API call to ${endpoint}`, params);
    }

    async logError(error, context = '') {
        const errorMessage = error instanceof Error ? error.message : String(error);
        const stack = error instanceof Error ? error.stack : '';

        await this.error(`Error${context ? ` in ${context}` : ''}: ${errorMessage}`, {
            stack: stack,
            error: error
        });
    }

    // Test logging function
    async test() {
        await this.debug('Logger test - debug message');
        await this.info('Logger test - info message');
        await this.warn('Logger test - warning message');
        await this.error('Logger test - error message');
    }
}

// Create singleton instance
const logger = new Logger();

// Export both the instance and the class
export { Logger, logger };
export default logger;