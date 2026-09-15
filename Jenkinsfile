/**
 * Aboriginal Art Gallery delivery pipeline.
 *
 * Build, Test, Code Quality, Security, Deploy (staging), Release (production,
 * behind an approval) and Monitoring (Datadog, with a simulated incident).
 * Setup and credentials: scripts/jenkins-setup.md.
 */

/** Records the running stage so post { failure } can name it. */
def enterStage() {
    env.LAST_STAGE = env.STAGE_NAME
}

pipeline {
    agent any

    options {
        timestamps()
        disableConcurrentBuilds()
        skipDefaultCheckout(true)
        timeout(time: 120, unit: 'MINUTES')
        buildDiscarder(logRotator(numToKeepStr: '30', artifactNumToKeepStr: '10'))
    }

    environment {
        CI = 'true'
        SQLX_OFFLINE = 'true'
        CARGO_TERM_COLOR = 'never'
        CARGO_BUILD_JOBS = '16'
        TEST_DB = "gallery-test-db-${env.BUILD_NUMBER}"
        STAGING_WEB_URL = 'http://host.docker.internal:8091'
        STAGING_API_URL = 'http://host.docker.internal:8081'
        PROD_WEB_URL = 'http://host.docker.internal:8092'
        PROD_API_URL = 'http://host.docker.internal:8082'
    }

    stages {
        stage('Checkout') {
            steps {
                enterStage()
                checkout scm
                script {
                    env.VERSION = sh(script: 'git rev-parse --short=8 HEAD', returnStdout: true).trim()
                    env.BUILD_TAG = "${env.BUILD_NUMBER}-${env.VERSION}"
                    env.RELEASE_VERSION = "v0.${env.BUILD_NUMBER}.0"
                    currentBuild.displayName = "#${env.BUILD_NUMBER} ${env.VERSION}"
                    currentBuild.description = "images :${env.BUILD_TAG}, release ${env.RELEASE_VERSION}"
                }
                sh 'rm -rf reports artifacts && mkdir -p reports/junit reports/security artifacts'
                echo "VERSION=${env.VERSION} BUILD_TAG=${env.BUILD_TAG} RELEASE_VERSION=${env.RELEASE_VERSION}"
            }
        }

        stage('Build') {
            parallel {
                stage('API') {
                    steps {
                        enterStage()
                        dir('api') {
                            sh 'cargo build --release --locked --bins'
                        }
                        script {
                            docker.build("gallery-api:${env.BUILD_TAG}", "--build-arg VERSION=${env.BUILD_TAG} api")
                        }
                        sh '''
                            docker save "gallery-api:${BUILD_TAG}" | gzip -1 > "artifacts/gallery-api-${BUILD_TAG}.tar.gz"
                            ls -lh artifacts/
                        '''
                    }
                }
                stage('Web') {
                    steps {
                        enterStage()
                        dir('web') {
                            sh 'pnpm install --frozen-lockfile'
                            sh 'pnpm build'
                        }
                        script {
                            docker.build("gallery-web:${env.BUILD_TAG}", "--build-arg VERSION=${env.BUILD_TAG} web")
                            docker.build("gallery-datadog-agent:${env.BUILD_TAG}", 'monitoring/datadog')
                        }
                    }
                }
            }
            post {
                success {
                    archiveArtifacts artifacts: 'web/dist/**, artifacts/*.tar.gz', fingerprint: true
                }
            }
        }

        stage('Test') {
            steps {
                enterStage()
                sh '''
                    docker run -d --name "$TEST_DB" --network gallery-ci \
                      -e POSTGRES_USER=gallery -e POSTGRES_PASSWORD=gallery -e POSTGRES_DB=gallery \
                      --health-cmd "pg_isready -U gallery -d gallery" --health-interval 2s --health-retries 30 \
                      postgis/postgis:16-3.4
                    for attempt in $(seq 60); do
                      [ "$(docker inspect -f '{{.State.Health.Status}}' "$TEST_DB")" = healthy ] && break
                      sleep 2
                    done
                    docker inspect -f 'test database {{.Name}} is {{.State.Health.Status}}' "$TEST_DB"
                '''
                withEnv(["DATABASE_URL=postgres://gallery:gallery@${env.TEST_DB}:5432/gallery"]) {
                    dir('api') {
                        sh 'sqlx migrate run && sqlx migrate info'
                    }
                    script {
                        parallel(
                            'Rust unit + integration': {
                                dir('api') {
                                    sh '''
                                        rm -rf "$CARGO_TARGET_DIR/nextest/ci" "$CARGO_TARGET_DIR/llvm-cov-target/nextest/ci"
                                        cargo ci-test
                                    '''
                                }
                            },
                            'Web components + stores': {
                                dir('web') {
                                    sh 'pnpm test:ci'
                                }
                            },
                            failFast: false
                        )
                    }
                }
            }
            post {
                always {
                    sh '''
                        junit_rust="$(find "$CARGO_TARGET_DIR" -path '*nextest/ci/junit.xml' -print -quit)"
                        [ -n "$junit_rust" ] && cp "$junit_rust" reports/junit/rust.xml || true
                        [ -f web/reports/junit.xml ] && cp web/reports/junit.xml reports/junit/web.xml || true
                    '''
                    junit testResults: 'reports/junit/rust.xml, reports/junit/web.xml', allowEmptyResults: true
                    recordCoverage id: 'rust-coverage', name: 'Rust coverage',
                        tools: [[parser: 'LCOV', pattern: 'api/target/lcov.info']],
                        sourceDirectories: [[path: 'api']]
                    recordCoverage id: 'web-coverage', name: 'Web coverage',
                        tools: [[parser: 'LCOV', pattern: 'web/coverage/lcov.info']],
                        sourceDirectories: [[path: 'web']]
                }
            }
        }

        stage('Code Quality') {
            steps {
                enterStage()
                dir('api') {
                    sh 'cargo fmt --check'
                }
                script {
                    def clippyStatus = sh(script: 'cd api && cargo ci-clippy > ../reports/clippy.json', returnStatus: true)
                    recordIssues id: 'clippy', name: 'Clippy', sourceDirectories: [[path: 'api']],
                        tool: cargo(pattern: 'reports/clippy.json'),
                        qualityGates: [[threshold: 1, type: 'TOTAL', criticality: 'FAILURE']]
                    if (clippyStatus != 0) {
                        error "cargo clippy -D warnings failed with exit code ${clippyStatus}"
                    }
                }
                withSonarQubeEnv('sonarqube') {
                    sh 'sonar-scanner -Dsonar.projectVersion="$BUILD_TAG" -Dsonar.scm.revision="$(git rev-parse HEAD)"'
                }
                timeout(time: 10, unit: 'MINUTES') {
                    waitForQualityGate abortPipeline: true
                }
            }
        }

        stage('Security') {
            parallel {
                stage('Rust dependencies') {
                    steps {
                        enterStage()
                        dir('api') {
                            sh '''#!/usr/bin/env bash
                                set -uo pipefail
                                rc=0
                                cargo audit --json > ../reports/security/cargo-audit.json || rc=1
                                jq -r '"cargo audit: \\(.vulnerabilities.count) vulnerabilities",
                                       (.vulnerabilities.list[] | "  \\(.advisory.id) \\(.package.name)@\\(.package.version): \\(.advisory.title)")' \
                                  ../reports/security/cargo-audit.json
                                cargo deny --format json check 2> ../reports/security/cargo-deny.json || rc=1
                                cargo deny check 2>&1 | tail -n 5
                                exit $rc
                            '''
                        }
                    }
                }
                stage('Web dependencies') {
                    steps {
                        enterStage()
                        dir('web') {
                            sh '''#!/usr/bin/env bash
                                set -uo pipefail
                                pnpm audit --json > ../reports/security/pnpm-audit.json
                                pnpm audit --audit-level=high
                            '''
                        }
                    }
                }
                stage('Images and IaC') {
                    steps {
                        enterStage()
                        sh '''#!/usr/bin/env bash
                            set -uo pipefail
                            rc=0
                            trivy image --download-db-only --quiet
                            scan() {
                              local name="$1" image="$2" gate="$3"
                              trivy image --quiet --skip-db-update --severity HIGH,CRITICAL \
                                --ignorefile .trivyignore.yaml --exit-code "$gate" \
                                --format json --output "reports/security/trivy-${name}.json" "$image"
                              local status=$?
                              trivy convert --format table "reports/security/trivy-${name}.json"
                              return $status
                            }
                            scan api "gallery-api:${BUILD_TAG}" 1 || rc=1
                            scan web "gallery-web:${BUILD_TAG}" 1 || rc=1
                            scan datadog-agent "gallery-datadog-agent:${BUILD_TAG}" 0
                            trivy fs --quiet --skip-db-update --scanners misconfig,secret --severity HIGH,CRITICAL \
                              --ignorefile .trivyignore.yaml --skip-dirs web/node_modules --exit-code 1 \
                              --format json --output reports/security/trivy-repo.json . || rc=1
                            trivy convert --format table reports/security/trivy-repo.json
                            exit $rc
                        '''
                    }
                }
            }
            post {
                always {
                    archiveArtifacts artifacts: 'reports/security/**', allowEmptyArchive: true
                }
            }
        }

        stage('Deploy') {
            steps {
                enterStage()
                withCredentials([file(credentialsId: 'env-staging', variable: 'STAGING_ENV')]) {
                    sh '''#!/usr/bin/env bash
                        set -euo pipefail
                        source scripts/lib/common.sh
                        source scripts/lib/deploy.sh
                        deploy_stack staging "$STAGING_ENV" "$BUILD_TAG"
                    '''
                    sh 'SMOKE_SUITE=smoke.staging scripts/smoke-test.sh "$STAGING_WEB_URL" "$STAGING_API_URL" "$STAGING_ENV" reports/junit/smoke-staging.xml'
                }
            }
            post {
                always {
                    junit testResults: 'reports/junit/smoke-staging.xml', allowEmptyResults: true
                }
            }
        }

        stage('Release') {
            steps {
                enterStage()
                timeout(time: 30, unit: 'MINUTES') {
                    input message: "Promote ${env.BUILD_TAG} to production as ${env.RELEASE_VERSION}?", ok: 'Promote'
                }
                sh '''
                    if git ls-remote --exit-code --tags "$GALLERY_REPO_URL" "refs/tags/$RELEASE_VERSION" >/dev/null; then
                      echo "tag $RELEASE_VERSION already exists on the remote" >&2
                      exit 1
                    fi
                '''
                withCredentials([
                    file(credentialsId: 'env-prod', variable: 'PROD_ENV_FILE'),
                    string(credentialsId: 'datadog-api-key', variable: 'DD_API_KEY'),
                ]) {
                    sh 'scripts/release.sh "$BUILD_TAG" "$RELEASE_VERSION" reports/release.json'
                }
                sshagent(credentials: ['github-deploy-key']) {
                    sh '''
                        git -c user.name="Jenkins CI" -c user.email="jenkins@gallery.local" \
                          tag -a "$RELEASE_VERSION" -m "Release $RELEASE_VERSION (build $BUILD_TAG)"
                        GIT_SSH_COMMAND="ssh -o StrictHostKeyChecking=accept-new" \
                          git push "$GALLERY_PUSH_URL" "refs/tags/$RELEASE_VERSION"
                    '''
                }
            }
            post {
                always {
                    junit testResults: 'reports/junit/smoke-production.xml', allowEmptyResults: true
                }
            }
        }

        stage('Monitoring') {
            steps {
                enterStage()
                withCredentials([
                    string(credentialsId: 'datadog-api-key', variable: 'DD_API_KEY'),
                    string(credentialsId: 'datadog-app-key', variable: 'DD_APP_KEY'),
                ]) {
                    sh '''#!/usr/bin/env bash
                        set -euo pipefail
                        scripts/datadog-apply.sh > reports/datadog-dashboard-url.txt
                        scripts/datadog-verify.sh "$RELEASE_VERSION"
                        echo "Datadog dashboard: $(cat reports/datadog-dashboard-url.txt)"
                    '''
                    sh 'scripts/simulate-incident.sh reports/incident.json'
                }
            }
        }
    }

    post {
        always {
            sh 'docker rm -f "$TEST_DB" >/dev/null 2>&1 || true'
            archiveArtifacts artifacts: 'reports/**', allowEmptyArchive: true
        }
        success {
            echo "Released ${env.RELEASE_VERSION} (${env.BUILD_TAG}). Rollback: scripts/rollback.sh"
        }
        failure {
            echo "Pipeline FAILED in stage '${env.LAST_STAGE}'. See that stage's log and the archived reports/."
        }
        aborted {
            echo "Pipeline ABORTED during stage '${env.LAST_STAGE}'."
        }
    }
}
