roadmap

🚀 Sugestões de expansão (10 novas funcionalidades)

Rollback automático

Se um deploy falhar (ex: healthcheck não passar), remover os containers/volumes/redes criados e restaurar a versão anterior.

Logs e monitoramento em tempo real

Comando CLI para logs <service> que conecta via SSH e segue logs (docker logs -f).

Integração com Prometheus/Grafana ou envio de métricas básicas.

Escalabilidade dinâmica (scale up/down)

Permitir deploy myservice --scale 3 para subir múltiplas réplicas do mesmo serviço.

Ajustar automaticamente balanceadores de carga (se definidos).

Gerenciamento de secrets

Suporte a docker secret ou arquivos .env criptografados (com GPG/AES).

Injetar secrets de forma segura no container.

Blue/Green e Canary deploy

Subir uma nova versão em paralelo, verificar health, e só depois migrar tráfego.

Canary: rodar só uma parte do tráfego no novo container antes de substituir tudo.

Suporte multi-host

Hoje o deploy parece focado em um host remoto.

Expandir para clusters de hosts com balanceamento entre eles.

Integração com repositórios de imagens (registry)

Ao invés de docker save + scp, puxar imagens diretamente de um Docker Registry privado, com autenticação.

Comando de status e ps

Ver containers rodando, status (running/exited), uptime e portas expostas.

Parecido com docker-compose ps.

Suporte a rollouts de configuração

Atualizar só variáveis de ambiente, volumes ou redes sem recriar tudo.

Reload de containers com docker update.

Integração com sistemas de CI/CD

Expor API (HTTP ou gRPC) para ser chamado por pipelines de build/deploy.

Exemplo: curl -X POST /deploy --data '{"group": "backend"}'.



🔹 10 funcionalidades voltadas para DevOps / Observabilidade

Comando status e painel de visualização

Mostrar quais serviços estão rodando, suas portas, uptime, versão da imagem, estado do healthcheck.

Logs centralizados

Capturar docker logs e enviar para Elasticsearch, Loki ou até salvar localmente com rotação automática.

Métricas de containers

Exportar métricas de CPU, memória, rede por container (via docker stats) em formato Prometheus.

Alertas de falhas

Notificar via Slack/Discord/Email se um serviço não passar no healthcheck ou cair.

Suporte a pipelines GitOps

Deploy automático sempre que um repositório Git for atualizado (ex: novo commit na branch main → redeploy).

Auditoria e histórico de deploys

Guardar logs de “quem deployou o quê” e permitir rollback para versões anteriores.

Suporte a múltiplos arquivos de configuração

Possibilidade de compor vários YAMLs (ex: base.yaml + prod.yaml) para ambientes diferentes.

CLI interativo estilo TUI

Criar uma interface em terminal (tipo htop) mostrando serviços em tempo real e permitindo operações (restart, scale, logs).

Controle de acesso (RBAC)

Definir permissões por usuário/grupo: quem pode criar rede, quem pode apenas restartar serviços, etc.

Testes de smoke/integração pós-deploy

Rodar automaticamente um conjunto de testes simples (ping de endpoint, execução de query no DB, etc.) após cada deploy.
