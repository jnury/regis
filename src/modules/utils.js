// Utility functions module - extracted for testability
export const Utils = {
    showElement(element) {
        if (element) {
            element.classList.remove('hidden');
        }
    },

    hideElement(element) {
        if (element) {
            element.classList.add('hidden');
        }
    },

    showLoading(message = 'Loading...') {
        const statusText = document.getElementById('status-text');
        const loadingSpinner = document.getElementById('loading-spinner');

        if (statusText) statusText.textContent = message;
        if (loadingSpinner) loadingSpinner.classList.remove('hidden');
    },

    hideLoading() {
        const loadingSpinner = document.getElementById('loading-spinner');
        const statusText = document.getElementById('status-text');

        if (loadingSpinner) loadingSpinner.classList.add('hidden');
        if (statusText) statusText.textContent = 'Ready';
    },

    showError(message, details = null) {
        const errorDiv = document.createElement('div');
        errorDiv.className = 'error-message';
        errorDiv.innerHTML = `
            <div>${message}</div>
            ${details ? `<div class="error-details">${details}</div>` : ''}
        `;

        // Insert at the beginning of main
        const main = document.querySelector('.main');
        if (main) {
            main.insertBefore(errorDiv, main.firstChild);

            // Remove after 5 seconds
            setTimeout(() => {
                if (errorDiv.parentNode) {
                    errorDiv.parentNode.removeChild(errorDiv);
                }
            }, 5000);
        }
    },

    showSuccess(message) {
        const successDiv = document.createElement('div');
        successDiv.className = 'success-message';
        successDiv.textContent = message;

        const main = document.querySelector('.main');
        if (main) {
            main.insertBefore(successDiv, main.firstChild);

            setTimeout(() => {
                if (successDiv.parentNode) {
                    successDiv.parentNode.removeChild(successDiv);
                }
            }, 3000);
        }
    }
};